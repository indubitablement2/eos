use parking_lot::RwLock;
use crate::time_manager::TimeManagerConfig;

static CONFIG: RwLock<Option<Config>> = RwLock::new(None);

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub battlescape_server_time_manager_config: TimeManagerConfig,
    pub battlescape_client_time_manager_config: TimeManagerConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            battlescape_server_time_manager_config: Default::default(),
            battlescape_client_time_manager_config: Default::default(),
        }
    }
}
impl Config {
    /// TODO: Try to load from file or use default values.
    pub fn load() {
        *CONFIG.write() = Some(Default::default());
    }

    /// TODO: Save to file.
    pub fn save() {

    }

    pub fn get() -> Config {
        CONFIG.read().unwrap_or_default()
    }
}