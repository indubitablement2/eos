#[macro_use]
extern crate log;

pub mod packets;

/// How long between each Battlescape/Metascape tick.
pub const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
/// The server udp/tcp port number.
pub const SERVER_PORT: u16 = 36188;