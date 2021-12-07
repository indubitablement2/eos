#[macro_use]
extern crate log;

pub mod packets;

pub const VERSION_MAJOR: u16 = 0;
pub const VERSION_MINOR: u16 = 1;
pub const VERSION_PATCH: u16 = 0;
/// How long between each Battlescape/Metascape tick.
pub const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
/// The server udp/tcp port number.
pub const SERVER_PORT: u16 = 36188;