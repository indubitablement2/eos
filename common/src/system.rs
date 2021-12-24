use std::f32::consts::TAU;
use glam::Vec2;
use serde::{Serialize, Deserialize};

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
    pub radius: f32,
    /// What is this body orbiting around.
    /// 
    /// If this body is orbiting around another
    /// (like a moon around its planet or a planet around a start),
    /// this is the index offset from the first body in the system of the other body.
    /// 
    /// Otherwise it is orbiting around the system center.
    pub parent: Option<u8>,
    /// The distance it is orbiting from its parent.
    pub orbit_radius: f32,
    /// How many tick does it take this body to complete an orbit.
    /// 
    /// This can be negative and will result in counter clockwise rotation.
    pub orbit_time: f32,
}
impl CelestialBody {
    /// Return the orbit rotation in radian at a specific instant.
    /// 
    /// Time is an f32 to allow more granularity. Otherwise `u32 as f32` will work just fine.
    pub fn get_orbit_rotation(&self, time: f32) -> f32 {
        (time / self.orbit_time) * TAU
    }

    /// Return the body's position relative to its parent(s).
    /// 
    /// Time is an f32 to allow more granularity. Otherwise `u32 as f32` will work just fine.
    pub fn get_body_relative_position(&self, time: f32) -> Vec2 {
        let rot = self.get_orbit_rotation(time);
        Vec2::new(f32::cos(rot), f32::sin(rot)) * self.orbit_radius
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    /// Edge of the outtermost `CelestialBody`.
    pub bound: f32,
    /// The center of this `System` in world space.
    pub position: Vec2,
    /// Bodies are ordered by parent -> child.
    /// The first body is never a child.
    pub bodies: Vec<CelestialBody>,
    /// Some infos like name and looks that do not affect the simulation.
    pub infos: Vec<CelestialBodyInfo>,
}
impl System {
    /// Return the position of all bodies in this in world space.
    /// 
    /// Result should be empty.
    pub fn get_bodies_position(&self, time: f32, result: &mut Vec<Vec2>) {
        result.reserve(self.bodies.len());
        for body in self.bodies.iter() {
            let mut body_pos = body.get_body_relative_position(time);
            if let Some(other_offset) = body.parent {
                body_pos += result[other_offset as usize];
            } else {
                body_pos += self.position;
            }
            result.push(body_pos);
        }
    }
}

/// Since this vec should never be modified at runtime, index are SystemId.
/// Systems can change from version to version however.
/// TODO: Add a way to update SystemId to a newer version.
/// 
/// Systems are sorted on the y axis from the top.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Systems(pub Vec<System>);
impl Systems {
    pub fn get_bound(&self) -> f32 {
        self.0.iter().fold(0.0f32, |acc, system| {
            acc.max(system.position.length() + system.bound)
        })
    }
}
impl Default for Systems {
    fn default() -> Self {
        Self(Vec::new())
    }
}
