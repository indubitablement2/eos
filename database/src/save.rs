use super::*;

impl Database {
    pub fn save(&self) {
        let save = DatabaseSave::V0;

        // TODO: Save save to file
    }

    pub fn load() -> Self {
        // TODO: Load save from file
        let mut database_save = DatabaseSave::default();
        loop {
            match database_save.to_database() {
                Ok(db) => return db,
                Err(save) => database_save = save,
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
enum DatabaseSave {
    #[default]
    V0,
    V1 {
        next_client_id: u64,
    },
}
impl DatabaseSave {
    fn to_database(self) -> Result<Database, Self> {
        Err(match self {
            DatabaseSave::V0 => Self::V1 { next_client_id: 1 },
            DatabaseSave::V1 { next_client_id } => {
                return Ok(Database {
                    password: std::env::var("DATABASE_PASSWORD").unwrap(),
                    restart_request: None,
                    connection_listener: ConnectionListener::bind(common::DATABASE_ADDRESS)
                        .unwrap(),
                    connections: Default::default(),
                    mutations: Default::default(),
                    next_server_id: Default::default(),
                    servers: Default::default(),
                    queued_simulations: Default::default(),
                    simulations: Default::default(),
                    next_ship_id: Default::default(),
                    ships: Default::default(),
                    next_client_id: ClientId::try_from_u64(next_client_id).unwrap(),
                    clients: Default::default(),
                    username: Default::default(),
                });
            }
        })
    }
}
