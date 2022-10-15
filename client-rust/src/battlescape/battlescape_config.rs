use super::time_manager::TimeManagerConfigs;

#[derive(Debug, Clone, Copy)]
pub struct BattlescapeConfig {
    pub server_time_manager_configs: TimeManagerConfigs,
    pub client_time_manager_configs: TimeManagerConfigs,
}
impl Default for BattlescapeConfig {
    fn default() -> Self {
        Self {
            server_time_manager_configs: Default::default(),
            client_time_manager_configs: Default::default(),
        }
    }
}
