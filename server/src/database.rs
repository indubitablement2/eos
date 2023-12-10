use crate::battlescape::{entity::EntitySave, BattlescapeMiscSave};

use super::*;
use chrono::{DateTime, FixedOffset, Utc};
use instance::ClientLogin;
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
pub enum DatabaseRequest {
    ClientAuth {
        login: ClientLogin,
        response_token: u64,
    },
    SaveBattlescape {
        battlescape_id: BattlescapeId,
        battlescape_misc_save: Vec<u8>,
    },
    SaveShip {
        ship_id: ShipId,
        entity_save: Vec<u8>,
    },
    MoveShip {
        ship_id: ShipId,
        battlescape_id: BattlescapeId,
        entity_save: Vec<u8>,
    },
    Query(DatabaseQuery),
}
impl Packet for DatabaseRequest {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
    }
}

/// Potentially expensive query which does not mutate the database.
/// Batched and processed in parallel.
#[derive(Serialize, Deserialize)]
pub enum DatabaseQuery {
    RemoveMe,
}

#[derive(Serialize, Deserialize)]
pub enum DatabaseResponse {
    ClientAuth {
        client_id: Option<ClientId>,
        response_token: u64,
    },
}
impl Packet for DatabaseResponse {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
    }
}

// ####################################################################################
// ############## DATABASE ############################################################
// ####################################################################################

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Database {
    is_json: bool,
    save_count: u64,

    #[serde(skip)]
    mut_requests_writer: BufWriter<File>,

    /// Id is unused.
    #[serde(skip)]
    connection_listener: ConnectionListener,
    #[serde(skip)]
    next_instance_id: InstanceId,
    // TODO: Which battlescapes is this running
    #[serde(skip)]
    instances: AHashMap<InstanceId, Instance>,

    battlescapes: AHashMap<BattlescapeId, Battlescape>,
    ships: AHashMap<ShipId, Ship>,

    next_client_id: ClientId,
    clients: AHashMap<ClientId, Client>,
    username: AHashMap<String, ClientId>,
}
struct Instance {
    connection: Connection,
    battlescapes: AHashSet<BattlescapeId>,
}
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Battlescape {
    /// [battlescape::BattlescapeMiscSave]
    battlescape_misc_save: Vec<u8>,
    #[serde(skip)]
    ships: AHashSet<ShipId>,
    // TODO: planets state
    // TODO: star?
    // TODO: battlescape connections
    // TODO: Position
}
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Ship {
    battlescape_id: BattlescapeId,
    /// [battlescape::entity::EntitySave]
    entity_save: Vec<u8>,
}
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Client {
    password: Option<String>,
}

// ####################################################################################
// ############## DEFAULT #############################################################
// ####################################################################################

impl Default for Database {
    fn default() -> Self {
        Self {
            is_json: false,
            save_count: Default::default(),
            mut_requests_writer: BufWriter::new(File::create("dummy").unwrap()),
            connection_listener: ConnectionListener::bind(database_addr(), DatabaseAuth),
            next_instance_id: Default::default(),
            instances: Default::default(),
            battlescapes: Default::default(),
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
    fn prepare_fresh(&mut self) -> anyhow::Result<()> {
        for (ship_id, ship) in self.ships.iter() {
            self.battlescapes
                .get_mut(&ship.battlescape_id)
                .context("Ship's battlescape not found")?
                .ships
                .insert(*ship_id);
        }

        Ok(())
    }

    fn to_json(&mut self) -> anyhow::Result<()> {
        if self.is_json {
            return Ok(());
        }
        self.is_json = true;

        for battlescapes in self.battlescapes.values_mut() {
            battlescapes.battlescape_misc_save =
                bin_to_json::<BattlescapeMiscSave>(&battlescapes.battlescape_misc_save)?;
        }

        for ship in self.ships.values_mut() {
            ship.entity_save = bin_to_json::<EntitySave>(&ship.entity_save)?;
        }

        Ok(())
    }

    fn to_bin(&mut self) -> anyhow::Result<()> {
        if !self.is_json {
            return Ok(());
        }
        self.is_json = false;

        for battlescapes in self.battlescapes.values_mut() {
            battlescapes.battlescape_misc_save =
                json_to_bin::<BattlescapeMiscSave>(&battlescapes.battlescape_misc_save)?;
        }

        for ship in self.ships.values_mut() {
            ship.entity_save = json_to_bin::<EntitySave>(&ship.entity_save)?;
        }

        Ok(())
    }
}

fn bin_to_json<'a, T: Deserialize<'a> + Serialize>(data: &'a [u8]) -> anyhow::Result<Vec<u8>> {
    Ok(serde_json::to_vec(&bincode_decode::<T>(data)?)?)
}
fn json_to_bin<'a, T: Deserialize<'a> + Serialize>(data: &'a [u8]) -> anyhow::Result<Vec<u8>> {
    Ok(bincode_encode(&serde_json::from_slice::<T>(data)?))
}

// ####################################################################################
// ############## LOAD ################################################################
// ####################################################################################

const DATABASE_SUFFIX: &'static str = "_database";
const DATABASE_JSON_SUFFIX: &'static str = ".json";
const DATABASE_BIN_SUFFIX: &'static str = ".bin";
const MUT_REQUESTS_FILE: &'static str = "database_mutations.bin";

fn load_database() -> anyhow::Result<Database> {
    // Find latest database file.
    let mut database_path: Option<(PathBuf, DateTime<FixedOffset>, bool)> = None;
    for entry in std::env::current_dir()?.read_dir()? {
        let entry = entry?;

        let file_name = entry.file_name();
        let file_name = file_name.to_str().context("file name not utf8")?;

        let (prefix, is_json) = if let Some(prefix) = file_name
            .strip_suffix(DATABASE_JSON_SUFFIX)
            .and_then(|file_name| file_name.strip_suffix(DATABASE_SUFFIX))
        {
            (prefix, true)
        } else if let Some(prefix) = file_name
            .strip_suffix(DATABASE_JSON_SUFFIX)
            .and_then(|file_name| file_name.strip_suffix(DATABASE_SUFFIX))
        {
            (prefix, false)
        } else {
            continue;
        };

        let date = chrono::DateTime::parse_from_rfc3339(prefix)?;
        if let Some((_, prev_date, _)) = database_path {
            if date > prev_date {
                database_path = Some((entry.path(), date, is_json));
            }
        } else {
            database_path = Some((entry.path(), date, is_json));
        }
    }

    // Load database.
    let mut db: Database = if let Some((path, _, is_json)) = database_path {
        log::info!("Loading database from {}", path.display());
        let mut reader = BufReader::new(File::open(path)?);

        if is_json {
            serde_json::from_reader(&mut reader)?
        } else {
            bincode::Options::deserialize_from(bincode::DefaultOptions::new(), &mut reader)?
        }
    } else {
        log::warn!("No database file found, creating new one");
        Database::default()
    };

    db.to_bin()?;

    db.prepare_fresh()?;

    // Apply saved requests.
    let path = std::env::current_dir()?.join(MUT_REQUESTS_FILE);
    if let Ok(file) = File::open(path) {
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
                db.handle_request(&request_buf, None, false)?;
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

// ####################################################################################
// ############## SAVE ################################################################
// ####################################################################################

impl Database {
    fn save(&mut self, as_json: bool) -> anyhow::Result<()> {
        self.save_count += 1;

        let path = std::env::current_dir()?.join(format!(
            "{}{}{}",
            Utc::now().to_rfc3339(),
            DATABASE_SUFFIX,
            if as_json {
                DATABASE_JSON_SUFFIX
            } else {
                DATABASE_BIN_SUFFIX
            }
        ));
        let mut writer = BufWriter::new(File::create(path)?);
        if as_json {
            self.to_json()?;
            serde_json::to_writer(&mut writer, self)?;
            self.to_bin()?;
        } else {
            bincode::Options::serialize_into(bincode::DefaultOptions::new(), &mut writer, self);
        }
        writer.flush()?;

        let path = std::env::current_dir()?.join(MUT_REQUESTS_FILE);
        self.mut_requests_writer = BufWriter::new(File::create(path)?);
        self.mut_requests_writer
            .write_all(&self.save_count.to_le_bytes())?;

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
        db.step();
    }
}

impl Database {
    fn step(&mut self) {
        // Get new instances.
        while let Some((connection, _)) = self.connection_listener.recv() {
            let id = self.next_instance_id.next();

            self.instances.insert(
                id,
                Instance {
                    connection,
                    battlescapes: AHashSet::new(),
                },
            );
        }

        // Gather instance requests to process later.
        let mut requests = Vec::new();
        for (&instance_id, instance) in self.instances.iter_mut() {
            while let Some(buf) = instance.connection.recv::<Vec<u8>>() {
                requests.push((buf, instance_id));
            }
        }

        // Handle requests.
        let queries = requests
            .into_iter()
            .filter_map(
                |(request, from)| match self.handle_request(&request, Some(from), true) {
                    Ok(query) => query.map(|query| (query, from)),
                    Err(err) => {
                        log::error!("Failed to handle request: {}", err);
                        None
                    }
                },
            )
            .collect::<Vec<_>>();

        // Handle queries in parallel.
        queries.into_par_iter().for_each(|(query, from)| {
            self.handle_query(query, from);
        });

        // TODO: Handle disconnect
        self.instances.retain(|_, instance| {
            if instance.connection.is_connected() {
                true
            } else {
                log::warn!("Instance disconnected");
                false
            }
        });

        // TODO: Balance instances load
    }
}

// ####################################################################################
// ############## HANDLE REQUEST ######################################################
// ####################################################################################

impl Database {
    #[inline]
    fn handle_request(
        &mut self,
        request: &[u8],
        from: Option<InstanceId>,
        can_save: bool,
    ) -> anyhow::Result<Option<DatabaseQuery>> {
        let mut save = false;

        match bincode_decode::<DatabaseRequest>(request)? {
            DatabaseRequest::ClientAuth {
                login,
                response_token,
            } => {
                let client_id = if login.new_account {
                    save = true;
                    self.handle_register(login)
                } else {
                    self.handle_login(login)
                };

                if let Some(from) = from {
                    self.instances[&from]
                        .connection
                        .queue(DatabaseResponse::ClientAuth {
                            client_id,
                            response_token,
                        });
                }
            }
            DatabaseRequest::SaveBattlescape {
                battlescape_id,
                battlescape_misc_save,
            } => {
                save = true;

                self.battlescapes
                    .get_mut(&battlescape_id)
                    .context("Battlescape not found")?
                    .battlescape_misc_save = battlescape_misc_save;
            }
            DatabaseRequest::SaveShip {
                ship_id,
                entity_save,
            } => {
                save = true;

                self.ships
                    .get_mut(&ship_id)
                    .context("Ship not found")?
                    .entity_save = entity_save;
            }
            DatabaseRequest::MoveShip {
                ship_id,
                battlescape_id,
                entity_save,
            } => {
                save = true;

                let ship = self.ships.get_mut(&ship_id).context("Ship not found")?;

                self.battlescapes
                    .get_mut(&ship.battlescape_id)
                    .context("Ship's previous battlescape not found")?
                    .ships
                    .remove(&ship_id);

                let new_battlescape = self
                    .battlescapes
                    .get_mut(&battlescape_id)
                    .context("Ship's new battlescape not found")?;
                ship.battlescape_id = battlescape_id;
                ship.entity_save = entity_save;
                new_battlescape.ships.insert(ship_id);

                // TODO: Notify instance which run this battlescape
            }
            DatabaseRequest::Query(query) => {
                return Ok(Some(query));
            }
        }

        if save && can_save {
            self.mut_requests_writer
                .write_all(&(request.len() as u32).to_le_bytes())?;
            self.mut_requests_writer.write_all(request)?;
        }

        Ok(None)
    }

    fn handle_query(&self, query: DatabaseQuery, from: InstanceId) {
        match query {
            DatabaseQuery::RemoveMe => {}
        }
    }

    fn handle_register(&mut self, login: ClientLogin) -> Option<ClientId> {
        if login.username.len() < 4
            || login.username.len() > 32
            || login.password.len() < 4
            || self.username.contains_key(&login.username)
        {
            return None;
        }

        let client_id = self.next_client_id.next();
        self.username.insert(login.username, client_id);

        self.clients.insert(
            client_id,
            Client {
                password: Some(login.password),
            },
        );

        Some(client_id)
    }

    fn handle_login(&self, login: ClientLogin) -> Option<ClientId> {
        // TODO: Add other login method here
        self.login_username_password(&login.username, &login.password)
    }

    fn login_username_password(&self, username: &str, password: &str) -> Option<ClientId> {
        let client_id = *self.username.get(username)?;
        if self.clients.get(&client_id)?.password.as_deref()? == password {
            Some(client_id)
        } else {
            None
        }
    }
}

// ####################################################################################
// ############## CONNECTION ##########################################################
// ####################################################################################

pub fn connect_to_database() -> Connection {
    loop {
        std::thread::sleep(Duration::from_millis(500));

        match Connection::connect(database_addr(), DatabaseAuth) {
            Ok((connection, _)) => return connection,
            Err(err) => {
                log::warn!("Failed to connect to database: {}", err);
            }
        }
    }
}

#[derive(Clone)]
struct DatabaseAuth;
impl Authentication for DatabaseAuth {
    async fn login_packet(&mut self) -> impl Packet {
        DatabaseLogin {
            private_key: PRIVATE_KEY,
        }
    }

    async fn verify_first_packet(&mut self, first_packet: Vec<u8>) -> anyhow::Result<u64> {
        let first_packet = DatabaseLogin::parse(first_packet)?;
        if first_packet.private_key != PRIVATE_KEY {
            anyhow::bail!("Wrong private key")
        }
        Ok(0)
    }
}

#[derive(Serialize, Deserialize)]
struct DatabaseLogin {
    private_key: [u8; 32],
}
impl Packet for DatabaseLogin {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
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
    bincode_decode::<Database>(&bincode_encode(&db)).unwrap();
    serde_json::from_slice::<Database>(&serde_json::to_vec(&db).unwrap()).unwrap();
}
