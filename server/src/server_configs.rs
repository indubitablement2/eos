use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ServerConfigs {
    pub connection_handler_configs: ConnectionHandlerConfigs,
    pub clients_manager_configs: ClientsManagerConfigs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConnectionHandlerConfigs {
    /// Maximum number of pending connections handled per tick.
    pub max_new_connection_per_update: u32,
}
impl Default for ConnectionHandlerConfigs {
    fn default() -> Self {
        Self { max_new_connection_per_update: 4 }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientsManagerConfigs {
    /// Bind using loop back address.
    pub local: bool,
    /// How many pendings connections before update is considered.
    pub min_pending_for_update: usize,
    /// How many times do we have to call `handle_pending_connections`
    /// before an update is done (sending queue size, checking disconnect).
    pub pendings_update_interval: u32,
}
impl Default for ClientsManagerConfigs {
    fn default() -> Self {
        Self { local: true, min_pending_for_update: 100, pendings_update_interval: 50 }
    }
}