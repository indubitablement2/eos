use super::*;
use chrono::{DateTime, FixedOffset, Utc};
use instance::ClientLoginType;
use rayon::prelude::*;
use simulation::{entity::EntitySave, SimulationSave};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

/// Save every 4 hours.
const SAVE_INTERVAL: Duration = Duration::from_secs(4 * 60 * 60);
const KEEP_DATABASE_FILES_AMOUNT: usize = 12;

#[derive(Serialize, Deserialize)]
pub enum DatabaseRequest {
    SaveAndRestart {
        save_json: bool,
    },
    ClientAuth {
        login: ClientLoginType,
        response_token: u64,
    },
    SaveSimulation {
        simulation_id: SimulationId,
        simulation_save: SimulationSave,
    },
    SaveShip {
        ship_id: ShipId,
        simulation_id: SimulationId,
        save: EntitySave,
    },
    DeleteShip {
        ship_id: ShipId,
    },
    Query(DatabaseQuery),
}
impl Packet for DatabaseRequest {
    fn serialize(self) -> Vec<u8> {
        bin_encode(self)
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bin_decode(&buf)
    }
}

/// Potentially expensive query which does not mutate the database.
/// Batched and processed in parallel.
#[derive(Serialize, Deserialize)]
pub enum DatabaseQuery {
    /// Will respond with [DatabaseSimulationResponse::ClientShips].
    ClientShips {
        client_id: ClientId,
        from: SimulationId,
    },
}

#[derive(Serialize, Deserialize)]
pub enum DatabaseResponse {
    ClientAuthResult {
        client_id: Option<ClientId>,
        response_token: u64,
    },
    HandleSimulation {
        simulation_id: SimulationId,
        simulation_save: SimulationSave,
    },
    SaveAllSystems,
    DatabaseSimulationResponse {
        to: SimulationId,
        response: DatabaseSimulationResponse,
    },
}
impl Packet for DatabaseResponse {
    fn serialize(self) -> Vec<u8> {
        bin_encode(self)
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bin_decode(&buf)
    }
}

#[derive(Serialize, Deserialize)]
pub enum DatabaseSimulationResponse {
    ClientShips {
        client_id: ClientId,
        /// vec of [ClientShip]
        client_ships: Vec<u8>,
    },
    ShipEntered {
        ship_id: ShipId,
        save: EntitySave,
    },
}

#[derive(Serialize)]
struct ClientShip {
    ship_id: ShipId,
    simulation_id: SimulationId,
    flags: u8,
}

// ####################################################################################
// ############## DATABASE ############################################################
// ####################################################################################

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Database {
    save_count: u64,
    #[serde(skip)]
    next_save: Instant,
    /// 0: false, 1: bin, 2: json,
    #[serde(skip)]
    save_request: u8,
    #[serde(skip)]
    restart_request: Option<Instant>,

    #[serde(skip)]
    mut_requests_writer: Option<BufWriter<File>>,

    #[serde(skip)]
    connection_listener: ConnectionListener<DatabaseLogin>,
    #[serde(skip)]
    instances: IndexMap<InstanceId, Instance, RandomState>,
    #[serde(skip)]
    queries: Vec<(DatabaseQuery, InstanceId)>,

    simulations: AHashMap<SimulationId, Simulation>,
    ships: AHashMap<ShipId, Ship>,

    next_client_id: ClientId,
    clients: AHashMap<ClientId, Client>,
    username: AHashMap<String, ClientId>,
}
struct Instance {
    connection: Connection,
}
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Simulation {
    simulation_save: SimulationSave,
    #[serde(skip)]
    ships: AHashSet<ShipId>,
}
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Ship {
    simulation_id: SimulationId,
    save: EntitySave,
}
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Client {
    password: Option<String>,
    #[serde(skip)]
    ships: AHashSet<ShipId>,
}

// ####################################################################################
// ############## DEFAULT #############################################################
// ####################################################################################

impl Default for Database {
    fn default() -> Self {
        Self {
            save_count: Default::default(),
            next_save: Instant::now(),
            save_request: 0,
            restart_request: None,
            mut_requests_writer: None,
            connection_listener: ConnectionListener::bind(data().database_addr).unwrap(),
            instances: Default::default(),
            queries: Default::default(),
            simulations: Default::default(),
            ships: Default::default(),
            next_client_id: Default::default(),
            clients: Default::default(),
            username: Default::default(),
        }
    }
}

// ####################################################################################
// ############## BULK MUTATION #######################################################
// ####################################################################################

impl Database {
    /// Checks that all data is valid.
    fn prepare(&mut self) {
        for (simulation_id, system_data) in data().systems.iter() {
            self.simulations.entry(*simulation_id).or_default();
        }

        for (username, client_id) in self.username.iter() {
            if !self.clients.contains_key(client_id) {
                log::error!(
                    "{} has {:?}, but it is not found. Adding default client",
                    username,
                    client_id
                );
                self.clients.insert(*client_id, Default::default());
            }
        }

        self.ships.retain(|ship_id, ship| {
            ship.save.verify();

            if let Some(simulation) = self.simulations.get_mut(&ship.simulation_id) {
                simulation.ships.insert(*ship_id);

                if let Some(owner) = ship.save.owner {
                    if let Some(client) = self.clients.get_mut(&owner) {
                        client.ships.insert(*ship_id);
                    } else {
                        log::error!(
                            "{:?} owner ({:?}) not found. Removing owner",
                            ship_id,
                            owner
                        );
                        ship.save.owner = None;
                    }
                }

                true
            } else {
                log::error!(
                    "{:?}'s simulation ({:?}) not found. Removing ship",
                    ship_id,
                    ship.simulation_id
                );
                false
            }
        });
    }
}

// ####################################################################################
// ############## LOAD ################################################################
// ####################################################################################

/// database_2021-09-18T18:00:00+00:00.bin
const DATABASE_PREFIX: &'static str = "database_";
const DATABASE_JSON_SUFFIX: &'static str = ".json";
const DATABASE_BIN_SUFFIX: &'static str = ".bin";
const MUT_REQUESTS_FILE: &'static str = "mutations.bin";

struct DatabaseSavePath {
    path: PathBuf,
    date: DateTime<FixedOffset>,
    json: bool,
}

/// Sorted by date (oldest -> newest).
fn database_save_files() -> anyhow::Result<Vec<DatabaseSavePath>> {
    let mut files: Vec<DatabaseSavePath> = Vec::new();

    for entry in std::env::current_dir()?.read_dir()? {
        let entry = entry?;

        let full_file_name = entry.file_name();
        let full_file_name = full_file_name.to_str().context("file name not utf8")?;
        let Some(file_name) = full_file_name.strip_prefix(DATABASE_PREFIX) else {
            continue;
        };

        let (date, json) = if let Some(file_name) = file_name.strip_suffix(DATABASE_JSON_SUFFIX) {
            (file_name, true)
        } else if let Some(file_name) = file_name.strip_suffix(DATABASE_BIN_SUFFIX) {
            (file_name, false)
        } else {
            // Only database save file should start with DATABASE_PREFIX.
            log::warn!("Unknown file name format: {}", full_file_name);
            continue;
        };

        let date = chrono::DateTime::parse_from_rfc3339(date)?;

        files.push(DatabaseSavePath {
            path: entry.path(),
            date,
            json,
        })
    }

    files.sort_by_key(|file| file.date);

    Ok(files)
}

fn load_database() -> anyhow::Result<Database> {
    // Load newest database.
    let mut db: Database = if let Some(save) = database_save_files()?.pop() {
        log::info!("Loading database from {}", save.path.display());
        let mut reader = BufReader::new(File::open(save.path)?);

        if save.json {
            serde_json::from_reader(&mut reader)?
        } else {
            let mut buf = vec![0; 4096];
            postcard::from_io((&mut reader, &mut buf)).map(|(db, _)| db)?
        }
    } else {
        log::warn!("No database file found. Creating default one");
        Database::default()
    };

    db.prepare();

    // Apply saved requests.
    if let Ok(file) = File::open(MUT_REQUESTS_FILE) {
        let mut reader = BufReader::new(file);
        let mut buf = [0; 8];
        if reader
            .read_exact(&mut buf)
            .is_ok_and(|_| u64::from_le_bytes(buf) == db.save_count)
        {
            let mut buf = [0; 4];
            let mut request_buf = Vec::new();
            while let Ok(()) = reader.read_exact(&mut buf) {
                let len = u32::from_le_bytes(buf);
                request_buf.resize(len as usize, 0);
                reader.read_exact(&mut request_buf)?;
                db.handle_request(&request_buf, None)?;
            }

            log::info!("Previous database mutations applied");
        } else {
            log::error!("Database mutations file found but is outdated");
        }
    } else {
        log::warn!("No database mutations file found");
    }

    db.save(false)?;

    Ok(db)
}

fn remove_database_files(keep_amount: usize) -> anyhow::Result<()> {
    let mut files = database_save_files()?;

    let num_remove = files.len().saturating_sub(keep_amount);
    for file in files.drain(..num_remove) {
        log::info!("Removing old database file: {}", file.path.display());
        std::fs::remove_file(file.path)?;
    }

    Ok(())
}

// ####################################################################################
// ############## SAVE ################################################################
// ####################################################################################

impl Database {
    fn save(&mut self, json: bool) -> anyhow::Result<()> {
        let now = Instant::now();
        log::info!("Saving database. Json: {}", json);

        self.save_count += 1;
        self.next_save = now + SAVE_INTERVAL;

        let mut writer = BufWriter::new(File::create(format!(
            "{}{}{}",
            DATABASE_PREFIX,
            Utc::now().to_rfc3339(),
            if json {
                DATABASE_JSON_SUFFIX
            } else {
                DATABASE_BIN_SUFFIX
            }
        ))?);
        if json {
            serde_json::to_writer(&mut writer, self)?;
        } else {
            postcard::to_io(&self, &mut writer)?;
        }
        writer.flush()?;

        let mut writer = BufWriter::new(File::create(MUT_REQUESTS_FILE)?);
        writer.write_all(&self.save_count.to_le_bytes())?;
        self.mut_requests_writer = Some(writer);

        log::info!("Database saved in {} seconds", now.elapsed().as_secs());

        Ok(())
    }
}

// ####################################################################################
// ############## MAIN LOOP ###########################################################
// ####################################################################################

pub fn _start() {
    let mut db = load_database().unwrap();

    let mut interval = interval::Interval::new(50, 50);
    loop {
        interval.step();
        if db.step() {
            break;
        }
    }
}

impl Database {
    /// Return if disconnected.
    fn step(&mut self) -> bool {
        // Get new instances.
        while let Some((connection, login)) = self.connection_listener.recv() {
            if login.database_key != data().database_key {
                log::debug!("Refused instance login: Invalid private key");
                continue;
            }

            // Send simulations to instance.
            for &simulation_id in data()
                .instances
                .get(&login.instance_id)
                .unwrap()
                .systems
                .iter()
            {
                let simulations = &self.simulations[&simulation_id];

                connection.queue(DatabaseResponse::HandleSimulation {
                    simulation_id,
                    simulation_save: simulations.simulation_save.clone(),
                });

                for &ship_id in simulations.ships.iter() {
                    let ship = self.ships.get(&ship_id).unwrap();

                    connection.queue(DatabaseResponse::DatabaseSimulationResponse {
                        to: simulation_id,
                        response: DatabaseSimulationResponse::ShipEntered {
                            ship_id,
                            save: ship.save.clone(),
                        },
                    });
                }
            }

            connection.flush();

            self.instances
                .insert(login.instance_id, Instance { connection });
        }

        let mut i = 0;
        while i < self.instances.len() {
            match self.instances[i].connection.recv::<Vec<u8>>() {
                Ok(request) => {
                    if let Err(err) = self.handle_request(&request, Some(i)) {
                        log::error!("Failed to handle request: {}", err);
                    }
                }
                Err(TryRecvError::Empty) => i += 1,
                Err(TryRecvError::Disconnected) => {
                    self.instances.swap_remove_index(i);
                }
            }
        }

        // Handle queries in parallel.
        self.queries.par_iter().for_each(|(query, from)| {
            if let Err(err) = self.handle_query(query, *from) {
                log::error!("Failed to handle query: {}", err);
            }
        });
        self.queries.clear();

        for instance in self.instances.values() {
            instance.connection.flush();
        }

        if self.next_save < Instant::now() {
            self.save_request = self.save_request.max(1);
        }

        if self.save_request > 0 && self.restart_request.is_none()
            || self.restart_request.is_some_and(|at| at < Instant::now())
        {
            if let Err(err) = self.save(self.save_request >= 2) {
                log::error!("Failed to save database: {}", err);
            }

            if let Err(err) = remove_database_files(KEEP_DATABASE_FILES_AMOUNT) {
                log::error!("Failed to remove old database files: {}", err);
            }

            self.restart_request.is_some()
        } else {
            false
        }
    }
}

// ####################################################################################
// ############## HANDLE REQUEST ######################################################
// ####################################################################################

impl Database {
    fn handle_request(&mut self, request: &[u8], from: Option<usize>) -> anyhow::Result<()> {
        let save = match bin_decode::<DatabaseRequest>(request)? {
            DatabaseRequest::SaveAndRestart { save_json } => {
                if save_json {
                    self.save_request = 2;
                } else {
                    self.save_request = self.save_request.max(1);
                }

                for instance in self.instances.values() {
                    instance.connection.queue(DatabaseResponse::SaveAllSystems);
                }

                self.restart_request = Some(Instant::now() + Duration::from_secs(60));

                false
            }
            DatabaseRequest::ClientAuth {
                login,
                response_token,
            } => {
                let client_id = match login {
                    ClientLoginType::LoginUsernamePassword { username, password } => {
                        if let Some(client_id) = self.username.get(&username) {
                            let client = self
                                .clients
                                .get_mut(client_id)
                                .context("Client not found, but username exist")?;

                            if client.password.as_deref() == Some(password.as_str()) {
                                Some(*client_id)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    ClientLoginType::RegisterUsernamePassword { username, password } => {
                        if username.len() < 4
                            || username.len() > 32
                            || password.len() < 4
                            || self.username.contains_key(&username)
                        {
                            None
                        } else {
                            let client_id = self.next_client_id.next();
                            self.username.insert(username, client_id);

                            self.clients.insert(
                                client_id,
                                Client {
                                    password: Some(password),
                                    ..Default::default()
                                },
                            );

                            Some(client_id)
                        }
                    }
                };

                if let Some(from) = from {
                    self.instances[from]
                        .connection
                        .queue(DatabaseResponse::ClientAuthResult {
                            client_id,
                            response_token,
                        });
                }

                true
            }
            DatabaseRequest::SaveSimulation {
                simulation_id,
                simulation_save,
            } => {
                self.simulations
                    .get_mut(&simulation_id)
                    .context("Simulation not found")?
                    .simulation_save = simulation_save;

                true
            }
            DatabaseRequest::SaveShip {
                ship_id,
                simulation_id,
                save,
            } => {
                let new_owner = save.owner;

                let ship = Ship {
                    simulation_id,
                    save,
                };

                let mut remove_old_owner = None;
                let mut add_new_owner = None;
                let mut remove_new_simulation = None;
                let mut add_new_simulation = None;

                if let Some(old_ship) = self.ships.insert(ship_id, ship) {
                    if old_ship.save.owner != new_owner {
                        remove_old_owner = old_ship.save.owner;
                        add_new_owner = new_owner;
                    }

                    if old_ship.simulation_id != simulation_id {
                        remove_new_simulation = Some(old_ship.simulation_id);
                        add_new_simulation = Some(simulation_id);
                    }
                } else {
                    add_new_owner = new_owner;
                    add_new_simulation = Some(simulation_id);
                }

                if let Some(client_id) = remove_old_owner {
                    self.clients
                        .get_mut(&client_id)
                        .context("Ship's previous owner not found")?
                        .ships
                        .remove(&ship_id);
                }
                if let Some(client_id) = add_new_owner {
                    self.clients
                        .get_mut(&client_id)
                        .context("Ship's new owner not found")?
                        .ships
                        .insert(ship_id);
                }

                if let Some(simulation_id) = remove_new_simulation {
                    self.simulations
                        .get_mut(&simulation_id)
                        .context("Ship's previous simulation not found")?
                        .ships
                        .remove(&ship_id);
                }
                if let Some(simulation_id) = add_new_simulation {
                    self.simulations
                        .get_mut(&simulation_id)
                        .context("Ship's new simulation not found")?
                        .ships
                        .insert(ship_id);

                    // Notify new simulation
                    if let Some(instance) = self
                        .instances
                        .get(&data().systems[&simulation_id].instance_id)
                    {
                        instance
                            .connection
                            .queue(DatabaseResponse::DatabaseSimulationResponse {
                                to: simulation_id,
                                response: DatabaseSimulationResponse::ShipEntered {
                                    ship_id,
                                    save: self.ships[&ship_id].save.clone(),
                                },
                            });
                    }
                }

                true
            }
            DatabaseRequest::DeleteShip { ship_id } => {
                if let Some(ship) = self.ships.remove(&ship_id) {
                    if let Some(owner) = ship.save.owner {
                        if let Some(client) = self.clients.get_mut(&owner) {
                            client.ships.remove(&ship_id);
                        }
                    }

                    if let Some(simulation) = self.simulations.get_mut(&ship.simulation_id) {
                        simulation.ships.remove(&ship_id);
                    }
                }

                true
            }
            DatabaseRequest::Query(query) => {
                if let Some(from) = from {
                    self.queries
                        .push((query, *self.instances.get_index(from).unwrap().0));
                }

                false
            }
        };

        if save {
            if let Some(writer) = &mut self.mut_requests_writer {
                writer.write_all(&(request.len() as u32).to_le_bytes())?;
                writer.write_all(request)?;
            }
        }

        Ok(())
    }

    fn handle_query(&self, query: &DatabaseQuery, from_instance: InstanceId) -> anyhow::Result<()> {
        match query {
            DatabaseQuery::ClientShips { client_id, from } => {
                let client = self.clients.get(client_id).context("Client not found")?;

                let client_ships = bin_encode(
                    client
                        .ships
                        .iter()
                        .map(|ship_id| {
                            let ship = self.ships.get(ship_id).unwrap();
                            ClientShip {
                                ship_id: *ship_id,
                                simulation_id: ship.simulation_id,
                                flags: 0,
                            }
                        })
                        .collect::<Vec<ClientShip>>(),
                );

                self.instances[&from_instance].connection.queue(
                    DatabaseResponse::DatabaseSimulationResponse {
                        to: *from,
                        response: DatabaseSimulationResponse::ClientShips {
                            client_id: *client_id,
                            client_ships,
                        },
                    },
                );
            }
        }

        Ok(())
    }
}

// ####################################################################################
// ############## CONNECTION ##########################################################
// ####################################################################################

pub fn connect_to_database(instance_id: InstanceId) -> Connection {
    loop {
        std::thread::sleep(Duration::from_millis(500));

        match Connection::connect(
            data().database_addr,
            DatabaseLogin {
                instance_id,
                database_key: data().database_key.clone(),
            },
        ) {
            Ok(connection) => return connection,
            Err(err) => {
                log::warn!("Failed to connect to database: {}", err);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct DatabaseLogin {
    instance_id: InstanceId,
    database_key: Vec<u8>,
}
impl Packet for DatabaseLogin {
    fn serialize(self) -> Vec<u8> {
        bin_encode(self)
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bin_decode(&buf)
    }
}

// ####################################################################################
// ############## TEST ################################################################
// ####################################################################################

#[test]
fn test_json_stability() {
    #[derive(Serialize, Deserialize)]
    enum A {
        ToRemove(String),
        B { to_remove: u64, to_rename: u64 },
    }

    #[derive(Serialize, Deserialize)]
    enum B {
        B {
            #[serde(alias = "to_remove")]
            renamed: u64,
            #[serde(default)]
            to_add: u64,
        },
    }

    serde_json::from_slice::<B>(
        &serde_json::to_vec(&A::B {
            to_remove: 1,
            to_rename: 2,
        })
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn test_database_serialization() {
    let db = Database::default();
    bin_decode::<Database>(&bin_encode(&db)).unwrap();
    serde_json::from_slice::<Database>(&serde_json::to_vec(&db).unwrap()).unwrap();
}
