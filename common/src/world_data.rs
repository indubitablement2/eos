use crate::{idx::*, orbit::RelativeOrbit, reputation::Reputation};
use ahash::AHashMap;
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum StarType {
    Star,
    BlackHole,
    Nebula,
}
impl StarType {
    pub const BLACK_HOLE_FORCE: f32 = 10.0;
    
    pub fn to_str(self) -> &'static str {
        match self {
            StarType::Star => "star",
            StarType::BlackHole => "black hole",
            StarType::Nebula => "nebula",
        }
    }
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Star {
    pub star_type: StarType,
    pub radius: f32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Planet {
    pub radius: f32,
    pub relative_orbit: RelativeOrbit,
    pub temperature: f32,
    pub faction: Option<FactionId>,
    pub population: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    /// Edge of the outtermost `CelestialBody` + `SYSTEM_PADDING`.
    pub bound: f32,
    /// The center of this `System` in world space.
    pub position: Vec2,
    pub star: Star,
    /// Bodies are ordered by inner -> outter.
    pub planets: Vec<Planet>,
}
impl System {
    /// Extra radius added after the edge of the outtermost body of a system.
    pub const PADDING: f32 = 18.0;

    /// Compute the temperature of bodies in this system.
    pub fn compute_temperature(&mut self) {

    }
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
    pub total_num_planet: usize,
    pub next_system_id: u32,
}
impl WorldData {
    /// Result is saved in `bound`.
    pub fn update_bound(&mut self) {
        self.bound = self.systems.iter().fold(0.0f32, |acc, (_, system)| {
            acc.max(system.position.length() + system.bound)
        });
    }

    /// Result is saved in `total_num_planet`.
    pub fn update_total_num_planet(&mut self) {
        self.total_num_planet = self.systems.values().fold(0, |acc, system| acc + system.planets.len())
    }
}
