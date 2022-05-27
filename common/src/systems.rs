use crate::{
    idx::*,
    orbit::RelativeOrbit,
};
use ahash::AHashMap;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use utils::*;

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
    #[serde(skip)]
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields, Columns, Components)]
pub struct System {
    /// Edge of the outtermost `CelestialBody` + `SYSTEM_PADDING`.
    pub radius: f32,
    /// The center of this `System` in world space.
    pub position: Vec2,
    pub star: Star,
    /// Bodies are ordered by inner -> outter.
    pub planets: Vec<Planet>,
}
impl System {
    /// Extra radius added after the edge of the outtermost body of a system.
    pub const PADDING: f32 = 20.0;

    /// Compute the temperature of bodies in this system.
    pub fn compute_temperature(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Systems {
    pub systems: AHashMap<SystemId, System>,
    pub next_system_id: SystemId,
    /// The furthest system bound from the world origin.
    pub bound: f32,
    pub total_num_planet: usize,
}
impl Systems {
    pub const BOUND_PADDING: f32 = 128.0;

    pub fn update_all(&mut self) {
        self.update_bound();
        self.update_total_num_planet();
        self.update_all_temperature();
    }

    /// Result is saved in `bound`.
    pub fn update_bound(&mut self) {
        self.bound = self
            .systems
            .values()
            .fold(0.0f32, |acc, system| acc.max(system.position.length() + system.radius)) + Self::BOUND_PADDING;
    }

    /// Result is saved in `total_num_planet`.
    pub fn update_total_num_planet(&mut self) {
        self.total_num_planet = self.systems.values().fold(0, |acc, system| acc + system.planets.len())
    }

    // Update planets computed temperature.
    pub fn update_all_temperature(&mut self) {
        for system in self.systems.values_mut() {
            system.compute_temperature()
        }
    }

    pub fn get_system_and_planet(&self, planet_id: PlanetId) -> Option<(&System, &Planet)> {
        if let Some(system) = self.systems.get(&planet_id.system_id) {
            let planet = &system.planets[planet_id.planets_offset as usize];
            Some((system, planet))
        } else {
            None
        }
        
    }
}
