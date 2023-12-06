use super::*;
use chrono::{DateTime, FixedOffset, Utc};
use instance_server::ClientLogin;
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
pub enum DatabaseRequest {
    Mut(DatabaseRequestMut),
    Ref(DatabaseRequestRef),
    Query(DatabaseRequestQuery),
}

/// Mutate the database.
/// Single threaded; Should be very cheap to process.
#[derive(Serialize, Deserialize)]
pub enum DatabaseRequestMut {
    NewClient { login: ClientLogin },
    // move ship to battlescape
    // notify client ship changes
}

/// Does not mutate the database, but mutate internal state.
/// Single threaded; Should be very cheap to process.
#[derive(Serialize, Deserialize, Clone)]
pub enum DatabaseRequestRef {
    ClientAuth { login: ClientLogin },
    // Subscribe to new ship in battlescape
    // unsubscibe
}

/// Potentially expensive non-mutable query.
/// Batched and processed in parallel.
#[derive(Serialize, Deserialize)]
pub enum DatabaseRequestQuery {
    TestRequest,
    // Query ship position
}

#[derive(Serialize, Deserialize)]
pub enum DatabaseResponse {
    ClientAuth {
        login: ClientLogin,
        client_id: Option<ClientId>,
    },
}

#[derive(Serialize, Deserialize, Default)]
struct Database {
    num_mut_requests: u64,

    battlescapes: AHashMap<BattlescapeId, ()>,

    next_client_id: ClientId,
    clients: AHashMap<ClientId, Client>,
    username: AHashMap<String, ClientId>,
    client_connection: AHashMap<ClientId, ()>,
}
#[derive(Serialize, Deserialize, Default)]
struct Client {
    password: Option<String>,
}

struct State {
    mut_requests_writer: BufWriter<File>,

    // Id is unused.
    connection_listener: ConnectionListener,
    next_instance_id: InstanceId,
    instances: AHashMap<InstanceId, Connection>,
}
impl State {
    const DATABASE_SUFFIX: &'static str = " database.json";
    const MUT_REQUESTS_FILE: &'static str = "database_mutations.bin";

    fn load() -> anyhow::Result<(Self, Database)> {
        let mut state = Self {
            mut_requests_writer: BufWriter::new(File::create("dummy")?),

            instances: AHashMap::new(),
            next_instance_id: InstanceId::default(),
            connection_listener: ConnectionListener::bind(database_addr(), DatabaseAuth),
        };

        // Find latest database file.
        let mut database_path: Option<(PathBuf, DateTime<FixedOffset>)> = None;
        let dir = std::env::current_dir()?;
        for entry in dir.read_dir()? {
            let entry = entry?;

            if let Some(prefix) = entry
                .file_name()
                .to_str()
                .and_then(|file_name| file_name.strip_suffix(Self::DATABASE_SUFFIX))
            {
                let date = chrono::DateTime::parse_from_rfc3339(prefix)?;
                if let Some((_, prev_date)) = database_path {
                    if date > prev_date {
                        database_path = Some((entry.path(), date));
                    }
                } else {
                    database_path = Some((entry.path(), date));
                }
            }
        }

        // Load database.
        let mut db: Database = if let Some((path, _)) = database_path {
            log::info!("Loading database from {}", path.display());
            let mut reader = BufReader::new(File::open(path)?);
            serde_json::from_reader(&mut reader)?
        } else {
            log::warn!("No database file found, creating new one");
            Database::default()
        };

        // Apply mut requests.
        let path = std::env::current_dir()?.join(Self::MUT_REQUESTS_FILE);
        if let Ok(file) = File::open(path) {
            let mut reader = BufReader::new(file);
            let mut buf = [0; 8];
            if reader
                .read_exact(&mut buf)
                .is_ok_and(|_| u64::from_le_bytes(buf) == db.num_mut_requests)
            {
                let mut buf = [0; 4];
                let mut request_buf = Vec::new();
                while let Ok(()) = reader.read_exact(&mut buf) {
                    let len = u32::from_le_bytes(buf);
                    request_buf.resize(len as usize, 0);
                    reader.read_exact(&mut request_buf)?;
                    state.handle_request_mut(&mut db, &request_buf, None);
                }

                log::info!("Previous database mutations applied");
            } else {
                log::warn!("Database mutations file found but is outdated");
            }
        } else {
            log::info!("No database mutations file found");
        }

        state.save(&db)?;

        Ok((state, db))
    }

    fn save(&mut self, db: &Database) -> anyhow::Result<()> {
        let path = std::env::current_dir()?.join(format!(
            "{}{}",
            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            Self::DATABASE_SUFFIX
        ));
        let mut writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer(&mut writer, db)?;
        writer.flush()?;

        let path = std::env::current_dir()?.join(Self::MUT_REQUESTS_FILE);
        self.mut_requests_writer = BufWriter::new(File::create(path)?);
        self.mut_requests_writer
            .write_all(&db.num_mut_requests.to_le_bytes())?;

        Ok(())
    }

    fn step(&mut self, db: &mut Database) {
        // Get new instances.
        while let Some((new_instance, _)) = self.connection_listener.recv() {
            let id = self.next_instance_id.next();

            self.instances.insert(id, new_instance);
        }

        // Gather instance requests.
        let mut requests = Vec::new();
        for (&instance_id, instance) in self.instances.iter_mut() {
            while let Some(buf) = instance.recv::<Vec<u8>>() {
                requests.push((instance_id, buf));
            }
        }

        // Handle requests.
        requests.retain(|(instance_id, buf)| match buf[0] {
            REQUEST_MUT_ID => {
                self.handle_request_mut(db, &buf[1..], Some(*instance_id));

                false
            }
            REQUEST_REF_ID => {
                self.handle_request_ref(db, &buf[1..], *instance_id);
                false
            }
            _ => true,
        });

        // Handle queries in parallel.
        requests.into_par_iter().for_each(|(instance_id, buf)| {
            self.handle_query(db, &buf[1..], instance_id);
        });

        // TODO: Handle disconnect
        self.instances.retain(|_, instance| {
            if instance.is_connected() {
                true
            } else {
                log::warn!("Instance disconnected");
                false
            }
        });

        // TODO: Balance instances load
    }

    fn handle_request_mut(&mut self, db: &mut Database, request: &[u8], from: Option<InstanceId>) {
        if let Err(err) = self
            .mut_requests_writer
            .write_all(&(request.len() as u32 - 1).to_le_bytes())
        {
            log::error!("Failed to write request length: {}", err);
            return;
        }
        if let Err(err) = self.mut_requests_writer.write_all(request) {
            log::error!("Failed to write request: {}", err);
            return;
        }
        db.num_mut_requests += 1;

        let request: DatabaseRequestMut = match bincode_decode(request) {
            Ok(request) => request,
            Err(err) => {
                log::error!("Failed to decode request mut: {}", err);
                return;
            }
        };

        match request {
            DatabaseRequestMut::NewClient { login } => {
                let client_id = if login.username.len() > 4
                    && login.username.len() < 32
                    && login.password.len() > 4
                    && db.username.contains_key(&login.username)
                {
                    None
                } else {
                    let client_id = db.next_client_id;
                    db.username.insert(login.username.clone(), client_id);

                    db.clients.insert(
                        client_id,
                        Client {
                            password: Some(login.password.clone()),
                        },
                    );

                    Some(client_id)
                };

                if let Some(from) = from {
                    self.instances[&from].queue(DatabaseResponse::ClientAuth { login, client_id });
                }
            }
        }
    }

    fn handle_request_ref(&mut self, db: &Database, request: &[u8], from: InstanceId) {
        let request: DatabaseRequestRef = match bincode_decode(request) {
            Ok(request) => request,
            Err(err) => {
                log::warn!("Failed to decode request ref: {}", err);
                return;
            }
        };

        match request {
            DatabaseRequestRef::ClientAuth { login } => {
                // TODO: Add other login method here

                let client_id = db.username.get(&login.username).and_then(|client_id| {
                    db.clients
                        .get(client_id)
                        .and_then(|client| client.password.as_deref())
                        .and_then(|password| {
                            if password == login.password.as_str() {
                                Some(*client_id)
                            } else {
                                None
                            }
                        })
                });

                self.instances[&from].queue(DatabaseResponse::ClientAuth { login, client_id });
            }
        }
    }

    fn handle_query(&self, db: &Database, request: &[u8], from: InstanceId) {
        let request: DatabaseRequestQuery = match bincode_decode(request) {
            Ok(request) => request,
            Err(err) => {
                log::warn!("Failed to decode request query: {}", err);
                return;
            }
        };

        match request {
            DatabaseRequestQuery::TestRequest => todo!(),
        }
    }
}

pub fn _start() {
    let (mut state, mut db) = State::load().unwrap();

    let mut interval = interval::Interval::new(50, 50);
    loop {
        interval.step();
        state.step(&mut db);
    }
}

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

impl Packet for DatabaseRequest {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
    }
}
impl Packet for DatabaseResponse {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
    }
}

const REQUEST_MUT_ID: u8 = 0;
const REQUEST_REF_ID: u8 = 1;

#[test]
fn test_request_encoding() {
    let request_mut = DatabaseRequestMut::NewClient {
        login: Default::default(),
    };
    let request_ref = DatabaseRequestRef::ClientAuth {
        login: Default::default(),
    };

    assert_eq!(
        &bincode_encode(&DatabaseRequest::Ref(request_ref.clone()))[1..],
        &bincode_encode(&request_ref)
    );

    let buf = bincode_encode(DatabaseRequest::Mut(request_mut));
    assert_eq!(buf[0], REQUEST_MUT_ID);

    let buf = bincode_encode(DatabaseRequest::Ref(request_ref));
    assert_eq!(buf[0], REQUEST_REF_ID);
}

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