use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::orbit_to_world_position;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Position {
    /// A standard position relative to the world origin.
    WorldPosition { world_position: Vec2 },
    /// An orbit around an arbitrary origin.
    Orbit {
        /// Origin in world space this position is orbiting aound.
        origin: Vec2,
        /// The distance it is orbiting from the origin.
        orbit_radius: f32,
        /// The initial angle.
        orbit_start_angle: f32,
        /// How long it takes to make a full orbit in tick.
        /// Negative value result in counter clockwise rotation.
        ///
        /// f32 is used to allow for more granularity.
        orbit_time: f32,
    },
}
impl Position {
    /// Return either the world position as is or compute it from the orbit.
    pub fn to_world_position(self, time: f32) -> Vec2 {
        match self {
            Position::WorldPosition { world_position } => world_position,
            Position::Orbit {
                origin,
                orbit_radius,
                orbit_start_angle,
                orbit_time,
            } => orbit_to_world_position(origin, orbit_radius, orbit_start_angle, orbit_time, time),
        }
    }
}
