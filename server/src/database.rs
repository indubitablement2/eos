use super::*;
use battlescape::{entity::EntitySave, BattlescapeMiscSave};
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
        battlescape_id: BattlescapeId,
        entity_save: Vec<u8>,
        owner: Option<ClientId>,
    },
    RemoveShip {
        ship_id: ShipId,
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
    /// Will respond with [DatabaseResponse::ClientShips] if client is online.
    ClientShips {
        client_id: ClientId,
        request: BattlescapeId,
    },
}

#[derive(Serialize, Deserialize)]
pub enum DatabaseResponse {
    ClientAuthResult {
        client_id: Option<ClientId>,
        response_token: u64,
    },
    HandleBattlescape {
        battlescape_id: BattlescapeId,
        battlescape_misc_save: Vec<u8>,
    },
    ClientShips {
        client_id: ClientId,
        request: BattlescapeId,
        /// vec of [ClientShip]
        client_ships: Vec<u8>,
    },
    ShipEntered {
        ship_id: ShipId,
        battlescape_id: BattlescapeId,
        entity_save: Vec<u8>,
        owner: Option<ClientId>,
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

#[derive(Serialize)]
struct ClientShip {
    ship_id: ShipId,
    battlescape_id: BattlescapeId,
    flags: u8,
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
    connection_listener: ConnectionListener<DatabaseLogin>,
    #[serde(skip)]
    next_instance_id: InstanceId,
    #[serde(skip)]
    instances: AHashMap<InstanceId, Instance>,
    #[serde(skip)]
    instance_inbounds: AHashMap<InstanceId, ConnectionInbound>,
    #[serde(skip)]
    queries: Vec<(DatabaseQuery, InstanceId)>,

    battlescapes: AHashMap<BattlescapeId, Battlescape>,
    ships: AHashMap<ShipId, Ship>,

    next_client_id: ClientId,
    clients: AHashMap<ClientId, Client>,
    username: AHashMap<String, ClientId>,
}
struct Instance {
    outbound: ConnectionOutbound,
    battlescapes: AHashSet<BattlescapeId>,
}
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Battlescape {
    /// [battlescape::BattlescapeMiscSave]
    battlescape_misc_save: Vec<u8>,
    #[serde(skip)]
    ships: AHashSet<ShipId>,
    #[serde(skip)]
    runner: Option<InstanceId>,
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
    owner: Option<ClientId>,
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
            is_json: false,
            save_count: Default::default(),
            mut_requests_writer: BufWriter::new(File::create("dummy").unwrap()),
            connection_listener: ConnectionListener::bind(database_addr()),
            next_instance_id: Default::default(),
            instances: Default::default(),
            instance_inbounds: Default::default(),
            queries: Default::default(),
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
    fn prepare_fresh(&mut self) {
        self.ships.retain(|ship_id, ship| {
            if let Some(battlescape) = self.battlescapes.get_mut(&ship.battlescape_id) {
                battlescape.ships.insert(*ship_id);

                if let Some(owner) = ship.owner {
                    if let Some(client) = self.clients.get_mut(&owner) {
                        client.ships.insert(*ship_id);
                    } else {
                        log::warn!("{:?} owner ({:?}) not found", ship_id, owner);
                        ship.owner = None;
                    }
                }

                true
            } else {
                log::warn!(
                    "{:?}'s battlescape ({:?}) not found",
                    ship_id,
                    ship.battlescape_id
                );
                false
            }
        });
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

    db.prepare_fresh();

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
            bincode::Options::serialize_into(bincode::DefaultOptions::new(), &mut writer, self)?;
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
        while let Some((connection, login)) = self.connection_listener.recv() {
            if &login.private_key != private_key() {
                log::debug!("Refused instance login: Invalid private key");
                continue;
            }

            let id = self.next_instance_id.next();
            let (outbound, inbound) = connection.split();

            self.instances.insert(
                id,
                Instance {
                    outbound,
                    battlescapes: AHashSet::new(),
                },
            );
            self.instance_inbounds.insert(id, inbound);
        }

        let mut instance_inbounds = std::mem::take(&mut self.instance_inbounds);
        instance_inbounds.retain(|&from, inbound| loop {
            match inbound.recv::<Vec<u8>>() {
                Ok(request) => {
                    if let Err(err) = self.handle_request(&request, Some(from), true) {
                        log::error!("Failed to handle request: {}", err);
                    }
                }
                Err(TryRecvError::Empty) => break true,
                Err(TryRecvError::Disconnected) => break false,
            }
        });
        self.instance_inbounds = instance_inbounds;

        // Handle queries in parallel.
        self.queries.par_iter().for_each(|(query, from)| {
            self.handle_query(query, *from);
        });
        self.queries.clear();

        // TODO: Handle disconnect
        self.instances.retain(|_, instance| {
            if instance.outbound.is_connected() {
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
    ) -> anyhow::Result<()> {
        let mut save = false;

        match bincode_decode::<DatabaseRequest>(request)? {
            DatabaseRequest::ClientAuth {
                login,
                response_token,
            } => {
                let client_id = match login {
                    ClientLogin::LoginUsernamePassword { username, password } => {
                        None
                        //
                    }
                    ClientLogin::RegisterUsernamePassword { username, password } => {
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

                // TODO: Add starting ship (if no ship)

                if let Some(from) = from {
                    self.instances[&from]
                        .outbound
                        .queue(DatabaseResponse::ClientAuthResult {
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
                battlescape_id,
                entity_save,
                owner,
            } => {
                save = true;

                let ship = self.ships.get_mut(&ship_id).context("Ship not found")?;

                ship.entity_save = entity_save;

                if ship.owner != owner {
                    if let Some(old_owner) = ship.owner {
                        self.clients
                            .get_mut(&old_owner)
                            .context("Ship's old owner not found")?
                            .ships
                            .remove(&ship_id);
                    }

                    if let Some(new_owner) = owner {
                        self.clients
                            .get_mut(&new_owner)
                            .context("Ship's new owner not found")?
                            .ships
                            .insert(ship_id);
                    }

                    ship.owner = owner;
                }

                if ship.battlescape_id != battlescape_id {
                    self.battlescapes
                        .get_mut(&ship.battlescape_id)
                        .context("Ship's previous battlescape not found")?
                        .ships
                        .remove(&ship_id);

                    let new_battlescape = self
                        .battlescapes
                        .get_mut(&battlescape_id)
                        .context("Ship's new battlescape not found")?
                        .ships
                        .insert(ship_id);

                    ship.battlescape_id = battlescape_id;

                    // TODO: Notify new battlescape
                }
            }
            DatabaseRequest::RemoveShip { ship_id } => {
                save = true;

                if let Some(ship) = self.ships.remove(&ship_id) {
                    if let Some(owner) = ship.owner {
                        if let Some(client) = self.clients.get_mut(&owner) {
                            client.ships.remove(&ship_id);
                        }
                    }

                    if let Some(battlescape) = self.battlescapes.get_mut(&ship.battlescape_id) {
                        battlescape.ships.remove(&ship_id);
                    }
                }
            }
            DatabaseRequest::Query(query) => {
                if let Some(from) = from {
                    self.queries.push((query, from));
                }
            }
        }

        if save && can_save {
            self.mut_requests_writer
                .write_all(&(request.len() as u32).to_le_bytes())?;
            self.mut_requests_writer.write_all(request)?;
        }

        Ok(())
    }

    fn handle_query(&self, query: &DatabaseQuery, from: InstanceId) -> anyhow::Result<()> {
        match query {
            DatabaseQuery::ClientShips { client_id, request } => {
                let client = self.clients.get(client_id).context("Client not found")?;

                let client_ships = bincode_encode(
                    client
                        .ships
                        .iter()
                        .map(|ship_id| {
                            let ship = self.ships.get(ship_id).unwrap();
                            ClientShip {
                                ship_id: *ship_id,
                                battlescape_id: ship.battlescape_id,
                                flags: 0,
                            }
                        })
                        .collect::<Vec<ClientShip>>(),
                );

                self.instances[&from]
                    .outbound
                    .queue(DatabaseResponse::ClientShips {
                        client_id: *client_id,
                        request: *request,
                        client_ships,
                    });
            }
        }

        Ok(())
    }

    fn register_username_password(
        &mut self,
        username: String,
        password: String,
    ) -> Option<ClientId> {
        if username.len() < 4
            || username.len() > 32
            || password.len() < 4
            || self.username.contains_key(&username)
        {
            return None;
        }

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

    fn login_username_password(&mut self, username: &str, password: &str) -> Option<ClientId> {
        let client_id = *self.username.get(username)?;
        let client = self.clients.get_mut(&client_id)?;

        if client.password.as_deref()? == password {
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

        match Connection::connect(
            database_addr(),
            DatabaseLogin {
                private_key: private_key().to_vec(),
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
    private_key: Vec<u8>,
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
    _DATABASE_ADDR.set("[::1]:0".parse().unwrap()).unwrap();

    let db = Database::default();
    bincode_decode::<Database>(&bincode_encode(&db)).unwrap();
    serde_json::from_slice::<Database>(&serde_json::to_vec(&db).unwrap()).unwrap();
}
