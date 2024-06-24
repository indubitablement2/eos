use super::*;
use common::{bin_decode, bin_encode_into};
use serde::{Deserialize, Serialize};

const SAVE_FILE_PATH: &str = "../../database_save";

#[derive(Debug, Serialize, Deserialize, Default)]
enum DatabaseSave {
    #[default]
    V0,
    V1 {
        next_client_id: ClientId,
        systems: HashMap<SystemId, ()>,
        next_ship_id: ShipId,
    },
}
impl DatabaseSave {
    fn to_database(self) -> Result<Database, Self> {
        Err(match self {
            DatabaseSave::V0 => Self::V1 {
                next_client_id: Default::default(),
                systems: Default::default(),
                next_ship_id: Default::default(),
            },
            DatabaseSave::V1 {
                next_client_id,
                systems,
                next_ship_id,
            } => {
                let mut servers = Vec::new();
                servers.resize_with(ServerId::data().len(), || None);

                let mut systems: HashMap<SystemId, System> = SystemId::systems_data_iter()
                    .map(|system| {
                        let system_id = SystemId(&system);
                        let mut system = System {
                            ships: Default::default(),
                        };

                        if let Some(data) = systems.get(&system_id) {
                            // TODO: use saved data
                        }

                        (system_id, system)
                    })
                    .collect();

                return Ok(Database {
                    password: std::env::var("DATABASE_PASSWORD").unwrap(),
                    next_save: Instant::now() + SAVE_INTERVAL,
                    restart_request: None,
                    connection_listener: ConnectionListener::bind(common::DATABASE_ADDRESS)
                        .unwrap(),
                    auth_connections: Default::default(),
                    servers,
                    systems,
                    next_ship_id,
                    ships: Default::default(),
                    next_client_id,
                    clients: Default::default(),
                    username: Default::default(),
                });
            }
        })
    }
    fn from_database(db: &Database) -> Self {
        DatabaseSave::V1 {
            next_client_id: db.next_client_id,
            systems: db.systems.iter().map(|(id, system)| (*id, ())).collect(),
            next_ship_id: db.next_ship_id,
        }
    }
}

impl Database {
    pub fn load() -> Database {
        let mut save: DatabaseSave = match std::fs::read(SAVE_FILE_PATH) {
            Ok(buf) => match bin_decode(&buf) {
                Ok(save) => save,
                Err(err) => {
                    log::error!("Failed to decode database save file: {}", err);
                    Default::default()
                }
            },
            Err(err) => {
                log::error!("Failed to open database save file: {}", err);
                Default::default()
            }
        };

        loop {
            match save.to_database() {
                Ok(db) => return db,
                Err(new_save) => save = new_save,
            }
        }
    }

    pub fn save(&self) {
        let temp_path = format!("{}_{}", SAVE_FILE_PATH, rand::random::<u64>());
        let file = match std::fs::File::create_new(&temp_path) {
            Ok(file) => file,
            Err(err) => {
                log::error!("Failed to create save file: {}", err);
                return;
            }
        };
        let file = std::io::BufWriter::new(file);
        bin_encode_into(DatabaseSave::from_database(self), file);
        if let Err(err) = std::fs::rename(temp_path, SAVE_FILE_PATH) {
            log::error!("Failed to rename temporary database save: {}", err);
        }
        log::info!("Database saved");
        // TODO: Send to backup storage.
    }
}
