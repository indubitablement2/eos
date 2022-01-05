use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::orbit_to_world_position;

/// Extra radius added after the edge of the outtermost body of a system.
pub const SYSTEM_PADDING: f32 = 20.0;

/// Infos that do not affect the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestialBodyInfo {
    pub seed: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CelestialBodyType {
    Star,
    Planet,
    BlackHole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestialBody {
    pub body_type: CelestialBodyType,
    /// The body's radius.
    pub radius: f32,
    /// The distance it is orbiting from the system center.
    pub orbit_radius: f32,
    pub orbit_start_angle: f32,
    /// How many tick does it take this body to complete an orbit.
    ///
    /// This can be negative and will result in counter clockwise rotation.
    pub orbit_time: f32,
}
impl CelestialBody {
    /// Return the body's world position.
    pub fn get_body_relative_position(&self, origin: Vec2, time: f32) -> Vec2 {
        orbit_to_world_position(origin, self.orbit_radius, self.orbit_start_angle, self.orbit_time, time)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    /// Edge of the outtermost `CelestialBody` + `SYSTEM_PADDING`.
    pub bound: f32,
    /// The center of this `System` in world space.
    pub position: Vec2,
    /// Bodies are ordered by inner -> outter.
    pub bodies: Vec<CelestialBody>,
    /// Some infos like name and looks that do not affect the simulation.
    pub infos: Vec<CelestialBodyInfo>,
}

/// Since this vec should never be modified at runtime, `SystemId` are index.
/// Systems can change from version to version however.
///
/// Systems are sorted on the y axis from the top.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Systems(pub Vec<System>);
impl Systems {
    /// The furthest system bound from the world origin.
    ///
    /// Don't use this at runtime as it is very expensive.
    pub fn get_bound(&self) -> f32 {
        self.0
            .iter()
            .fold(0.0f32, |acc, system| acc.max(system.position.length() + system.bound))
    }
}
