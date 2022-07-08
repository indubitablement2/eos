use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub struct Configs {
    pub connection_configs: ConnectionConfigs,
    pub metascape_configs: MetascapeConfigs,
    pub client_configs: ClientConfigs,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MetascapeConfigs {
    /// When breaking, acceleration is multiplied by this.
    pub break_acceleration_multiplier: f32,
    /// Object can never go above this speed.
    pub absolute_max_speed: f32,
    /// Add a static amount to systems's bound.
    pub systems_bound_padding: f32,
}
impl Default for MetascapeConfigs {
    fn default() -> Self {
        Self {
            break_acceleration_multiplier: 1.5,
            absolute_max_speed: 2.0,
            systems_bound_padding: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientConfigs {
    /// How often is the client's detected fleet list updated.
    /// If it does not update for a tick, entity are assumed to still be detected.
    pub client_detected_entity_update_interval: u32,
    /// Client can only detect at most that many new fleet per interval.
    pub max_new_detected_fleet_per_interval: usize,
    /// Client can not detect more than that many fleet.
    pub max_detected_fleet: usize,
}
impl Default for ClientConfigs {
    fn default() -> Self {
        Self {
            client_detected_entity_update_interval: 5,
            max_new_detected_fleet_per_interval: 40,
            max_detected_fleet: 400,
        }
    }
}
