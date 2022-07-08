use crate::{idx::*, orbit::RelativeOrbit};
use ahash::AHashMap;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use utils::acc::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum StarType {
    Star,
    BlackHole,
    Nebula,
}
impl StarType {
    pub const BLACK_HOLE_FORCE: f32 = 10.0;

    pub const fn to_str(self) -> &'static str {
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Planet {
    pub radius: f32,
    pub relative_orbit: RelativeOrbit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    /// Edge of the outtermost `CelestialBody` + (normaly) some padding.
    pub radius: f32,
    /// The center of this `System` in world space.
    pub position: Vec2,
    pub star: Star,
    /// Bodies are ordered by inner -> outter.
    pub planets: Vec<Planet>,
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
    pub fn update_all(&mut self) {
        self.update_bound();
        self.update_total_num_planet();
    }

    /// Result is saved in `bound`.
    pub fn update_bound(&mut self) {
        self.bound = self
            .systems
            .values()
            .fold(0.0f32, |acc, system| acc.max(system.position.length() + system.radius));
    }

    /// Result is saved in `total_num_planet`.
    pub fn update_total_num_planet(&mut self) {
        self.total_num_planet = self.systems.values().fold(0, |acc, system| acc + system.planets.len())
    }

    pub fn get_system_and_planet(&self, planet_id: PlanetId) -> Option<(&System, &Planet)> {
        self.systems.get(&planet_id.system_id).and_then(|system| {
            system.planets.get(planet_id.planets_offset as usize).map(|planet| (system, planet))
        })
    }

    pub fn create_acceleration_structure(&self) -> AccelerationStructure<Circle, SystemId> {
        let mut acc = AccelerationStructure::new();
        acc.extend(
            self.systems
                .iter()
                .map(|(system_id, system)| (Circle::new(system.position, system.radius), *system_id)),
        );
        acc.update();
        acc
    }
}
