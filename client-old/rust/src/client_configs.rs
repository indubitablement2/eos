use crate::time_manager::TimeManagerConfigs;

#[derive(Debug, Clone, Copy)]
pub struct ClientConfigs {
    pub system_draw_distance: f32,

    pub time_manager_configs: TimeManagerConfigs,
}
impl Default for ClientConfigs {
    fn default() -> Self {
        Self {
            system_draw_distance: 256.0,
            time_manager_configs: Default::default(),
        }
    }
}
