use crate::orbit::Orbit;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Extra radius added after the edge of the outtermost body of a system.
pub const SYSTEM_PADDING: f32 = 20.0;

/// Infos that do not affect the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestialBodyInfo {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
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
    pub orbit: Orbit,
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
pub struct Systems{
    pub systems: Vec<System>,
}
impl Systems {
    /// The furthest system bound from the world origin.
    ///
    /// Don't use this at runtime as it is very expensive.
    pub fn get_bound(&self) -> f32 {
        self.systems
            .iter()
            .fold(0.0f32, |acc, system| acc.max(system.position.length() + system.bound))
    }
}
