use super::*;
use crate::time_manager::TimeManagerConfig;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientConfig {
    pub log_level: u8,

    pub system_draw_distance: f32,

    pub metascape_time_manager_config: TimeManagerConfig,
    pub battlescape_time_manager_config: TimeManagerConfig,
}
impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            log_level: 5,
            system_draw_distance: 256.0,
            metascape_time_manager_config: Default::default(),
            battlescape_time_manager_config: Default::default(),
        }
    }
}
