use crate::time_manager::TimeManagerConfigs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ClientConfig {
    pub graphic_config: GraphicConfig,
    pub time_manager_configs: TimeManagerConfigs,
}
impl ClientConfig {
    pub fn load() -> Self {
        // TODO: Load from file.
        Self::default()
    }

    pub fn save(&self) {
        // TODO: Save to file.
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GraphicConfig {
    pub vsync: bool,
    pub fullscreen: bool,
    pub window_width: u32,
    pub window_height: u32,
}
impl Default for GraphicConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            window_width: 800,
            window_height: 600,
        }
    }
}
