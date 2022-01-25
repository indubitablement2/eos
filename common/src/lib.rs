#![feature(test)]
#![feature(duration_constants)]
#![feature(derive_default_enum)]

use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[macro_use]
extern crate log;

pub mod array_difference;
pub mod connection;
pub mod factions;
pub mod idx;
pub mod intersection;
pub mod orbit;
pub mod packets;
pub mod metascape_configs;
pub mod reputation;
pub mod systems;
pub mod tcp_loops;
pub mod time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    major: u16,
    minor: u16,
    patch: u16,
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
impl Version {
    /// The current app version.
    pub const CURRENT: Version = Version {
        major: 0,
        minor: 1,
        patch: 0,
    };
}

/// How long between each Battlescape/Metascape tick.
pub const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
/// The server's tcp port.
pub const SERVER_PORT: u16 = 31415;

/// The maximum distance to the center.
pub const WORLD_BOUND: f32 = u16::MAX as f32;
