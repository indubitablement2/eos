use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfigs {
    pub connection_configs: ConnectionConfigs,
    pub metascape_configs: MetascapeConfigs,
    pub performance_configs: PerformanceConfigs,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ConnectionConfigs {
    /// Bind using loop back address.
    pub local: bool,
    /// Maximum number of pending connections handled per tick.
    pub max_connection_handled_per_update: usize,
    /// How many pendings connections before a queue update is considered.
    pub min_pending_queue_size_for_update: usize,
    /// How many tick does the connection queue  need to be above `min_pending_queue_size_for_update`
    /// before an update is done (sending queue size, checking disconnect).
    pub connection_queue_update_interval: u32,
}
impl Default for ConnectionConfigs {
    fn default() -> Self {
        Self {
            local: true,
            max_connection_handled_per_update: 32,
            min_pending_queue_size_for_update: 100,
            connection_queue_update_interval: 50,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MetascapeConfigs {
    /// Multiply velocity every tick.
    pub friction: f32,
}
impl Default for MetascapeConfigs {
    fn default() -> Self {
        Self { friction: 0.95 }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerformanceConfigs {
    /// How often is the client's detected entity list updated.
    /// If it does not update for a tick, entity are assumed to still be detected.
    pub client_detected_entity_update_interval: u32,
}
impl Default for PerformanceConfigs {
    fn default() -> Self {
        Self {
            client_detected_entity_update_interval: 5,
        }
    }
}
