#![feature(test)]

use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[macro_use]
extern crate log;

pub mod array_difference;
pub mod collider;
pub mod generation;
pub mod idx;
pub mod packets;
pub mod parameters;
pub mod res_time;
pub mod system;

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
/// The server udp/tcp port number.
pub const SERVER_PORT: u16 = 36188;
