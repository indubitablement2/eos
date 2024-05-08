use super::*;
use common::{bin_decode, bin_encode};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

const NUM_SAVE_BACKUPS: usize = 100;
const SAVE_INTERVAL: Duration = Duration::from_secs(1 * 60 * 60);
const SAVE_FOLDER_PATH: &str = "../../database_save/";

fn get_sorted_save_files() -> Vec<PathBuf> {
    let mut ret: Vec<PathBuf> = match std::fs::read_dir(SAVE_FOLDER_PATH) {
        Ok(iter) => iter.filter_map(|v| v.ok().map(|v| v.path())).collect(),
        Err(err) => {
            log::error!("Failed to read save folder: {}", err);
            return vec![];
        }
    };

    ret.sort_by_cached_key(|path| {
        path.file_name()
            .map(|s| s.to_string_lossy().parse::<i64>().unwrap_or(i64::MIN))
            .unwrap_or(i64::MIN)
    });

    ret
}

impl Database {
    pub fn handle_save(&mut self) {
        if let Some(handle) = &self.save_in_progress {
            if handle.is_finished() {
                self.save_in_progress = None;
            }
        }

        if Instant::now()
            .checked_duration_since(self.next_save)
            .is_some()
            && self.save_in_progress.is_none()
        {
            self.save();
        }
    }

    pub fn save(&mut self) {
        let save = self.to_save();

        if let Some(handle) = self.save_in_progress.take() {
            let _ = handle.join();
        }

        self.save_in_progress = Some(std::thread::spawn(move || {
            let buf = bin_encode(&save);
            let name = match std::time::UNIX_EPOCH.elapsed() {
                Ok(duration) => duration.as_secs() as i64,
                Err(err) => err.duration().as_secs() as i64 * -1,
            };
            if let Err(err) = std::fs::create_dir_all(SAVE_FOLDER_PATH) {
                log::error!("Failed to create save folder: {}", err);
            }
            if let Err(err) = std::fs::write(format!("{}{}", SAVE_FOLDER_PATH, name), buf) {
                log::error!("Failed to save database: {}", err);
            }

            log::info!("Saved database to {}", name);

            let mut save_files = get_sorted_save_files();
            while save_files.len() > NUM_SAVE_BACKUPS {
                if let Err(err) = std::fs::remove_file(save_files.remove(0)) {
                    log::error!("Failed to remove old save file: {}", err);
                    break;
                }
            }
        }));

        self.next_save = Instant::now() + SAVE_INTERVAL;
    }

    pub fn load() -> Self {
        let mut database_save = if let Some(path) = get_sorted_save_files().last() {
            log::info!("Loading save file: {:?}", path);
            let buf = std::fs::read(path).unwrap();
            bin_decode(&buf).unwrap()
        } else {
            log::warn!("No save file found, starting new database");
            DatabaseSave::default()
        };

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
        next_client_id: ClientId,
        simulation_saves: Vec<(SystemId, Option<Vec<u8>>)>,
        next_ship_id: ShipId,
    },
}
impl DatabaseSave {
    fn to_database(self) -> Result<Database, Self> {
        Err(match self {
            DatabaseSave::V0 => Self::V1 {
                next_client_id: Default::default(),
                simulation_saves: Default::default(),
                next_ship_id: Default::default(),
            },
            DatabaseSave::V1 {
                next_client_id,
                simulation_saves,
                next_ship_id,
            } => {
                let mut servers = Vec::new();
                servers.resize_with(ServerId::data().len(), || None);

                let mut systems: HashMap<SystemId, System> = SystemId::systems_data_iter()
                    .map(|system| {
                        (
                            SystemId(&system),
                            System {
                                simulation_save: None,
                                ships: Default::default(),
                            },
                        )
                    })
                    .collect();
                for (system_id, save) in simulation_saves {
                    if let Some(system) = systems.get_mut(&system_id) {
                        system.simulation_save = save;
                    }
                }

                return Ok(Database {
                    password: std::env::var("DATABASE_PASSWORD").unwrap(),
                    next_save: Instant::now() + SAVE_INTERVAL,
                    save_in_progress: None,
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
}
impl Database {
    fn to_save(&self) -> DatabaseSave {
        DatabaseSave::V1 {
            next_client_id: self.next_client_id,
            simulation_saves: self
                .systems
                .iter()
                .map(|(id, system)| (*id, system.simulation_save.clone()))
                .collect(),
            next_ship_id: self.next_ship_id,
        }
    }
}
