use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfigs {
    pub connection_configs: ConnectionConfigs,
    pub metascape_configs: MetascapeConfigs,
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
    /// The maximum distance to the world's center.
    pub bound: f32,
}
impl Default for MetascapeConfigs {
    fn default() -> Self {
        Self {
            friction: 0.95,
            bound: u16::MAX as f32,
        }
    }
}
