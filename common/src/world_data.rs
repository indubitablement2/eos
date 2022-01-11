use crate::{idx::*, orbit::Orbit, reputation::Reputation};
use ahash::AHashMap;
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
    /// The highest `FactionId` has the reputation of all lower `FactionId`.
    ///
    /// eg: fleet a has faction `2` and fleet b has faction `4`.
    ///
    /// Relation = `faction[4].reputation[2]`.
    pub faction_relation: AHashMap<FactionId, Reputation>,
    /// Fallback reputation.
    pub default_reputation: Reputation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldData {
    pub systems: AHashMap<SystemId, System>,
    pub factions: AHashMap<FactionId, Faction>,
    /// The furthest system bound from the world origin.
    pub bound: f32,
    pub next_system_id: u32,
}
impl WorldData {
    /// Result is saved in `bound`.
    pub fn update_bound(&mut self) {
        self.bound = self.systems.iter().fold(0.0f32, |acc, (_, system)| {
            acc.max(system.position.length() + system.bound)
        });
    }
}
