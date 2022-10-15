use crate::time_manager::TimeManagerConfigs;

#[derive(Debug, Clone, Copy)]
pub struct GodotClientConfig {
    pub system_draw_distance: f32,

    pub server_metascape_time_manager_configs: TimeManagerConfigs,
    pub client_metascape_time_manager_configs: TimeManagerConfigs,
}
impl Default for GodotClientConfig {
    fn default() -> Self {
        Self {
            system_draw_distance: 256.0,
            server_metascape_time_manager_configs: Default::default(),
            client_metascape_time_manager_configs: Default::default(),
        }
    }
}
