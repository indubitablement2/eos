use crate::{idx::*, orbit::Orbit, reputation::Reputation};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Extra radius added after the edge of the outtermost body of a system.
pub const SYSTEM_PADDING: f32 = 20.0;

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

/// Infos that do not affect the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestialBodyInfo {
    pub name: String,
}

/// Infos that do not affect the simulation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColonyInfo {
    pub faction: Option<FactionId>,
    pub population: u64,
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
    pub colony: Vec<ColonyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub display_name: String,
    pub capital: Option<CelestialBodyId>,
    pub colonies: Vec<CelestialBodyId>,
    /// Reputation between factions.
    /// The lower FactionId is used here.
    /// 
    /// eg: fleet `a` is in faction `2` and fleet `b` has faction `4`.
    /// Relation = faction index `4` -> reputation index `2`.
    pub faction_relation: Vec<Reputation>,
    /// Fallback reputation.
    pub default_reputation: Reputation,
}

/// Since these vec should never be modified at runtime, idx are index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldData {
    /// sorted on the y axis from the top.
    pub systems: Vec<System>,
    pub factions: Vec<Faction>,
}
impl WorldData {
    /// The furthest system bound from the world origin.
    /// # Warning
    /// Don't use this at runtime as it is very expensive.
    /// It is there for tools and debugs.
    pub fn get_systems_bound(&self) -> f32 {
        self.systems
            .iter()
            .fold(0.0f32, |acc, system| acc.max(system.position.length() + system.bound))
    }
}
