#![feature(test)]
#![feature(duration_constants)]

use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[macro_use]
extern crate log;

pub mod array_difference;
pub mod connection;
pub mod fleet_movement;
pub mod idx;
pub mod intersection;
pub mod packets;
pub mod parameters;
pub mod position;
pub mod res_time;
pub mod system;
pub mod tcp_loops;
pub mod udp_loops;

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
/// The server's tcp/udp port.
pub const SERVER_PORT: u16 = 31415;

/// Return the world position of an orbit.
///
/// Time is an f32 to allow more granularity than tick. Otherwise `u32 as f32` will work just fine.
pub fn orbit_to_world_position(
    origin: Vec2,
    orbit_radius: f32,
    orbit_start_angle: f32,
    orbit_time: f32,
    time: f32,
) -> Vec2 {
    let rot = (time / orbit_time).mul_add(std::f32::consts::TAU, orbit_start_angle);
    Vec2::new(rot.cos(), rot.sin()) * orbit_radius + origin
}
