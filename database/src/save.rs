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

        if self
            .next_save
            .checked_duration_since(Instant::now())
            .is_some()
            && self.save_in_progress.is_none()
        {
            self.save();
        }
    }

    pub fn save(&mut self) {
        let save = DatabaseSave::V1 {
            next_client_id: self.next_client_id.to_u64(),
        };

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
                    next_save: Instant::now() + SAVE_INTERVAL,
                    save_in_progress: None,
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
