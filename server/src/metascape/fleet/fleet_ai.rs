use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum FleetAi {
    #[default]
    Idle,
    /// Fleet is controlled by a client.
    /// Actions will be taken from the client.
    // TODO: Fallback to what when client is not connected?
    ClientControl,
}
