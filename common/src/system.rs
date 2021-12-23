use std::f32::consts::TAU;
use glam::Vec2;

/// Infos that do not affect the simulation.
pub struct CelestialBodyInfo {
    pub seed: u64,
    pub name: String,
}

pub enum CelestialBodyType {
    Star,
    Planet,
    BlackHole,
}

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

pub struct System {
    /// Edge of the outtermost `CelestialBody`.
    pub bound: f32,
    /// The center of this `System` in world space.
    pub position: Vec2,
    /// The index of first body in this system.
    /// Bodies are ordered by parent -> child.
    /// The first body can not be a child.
    pub first_body: u32,
    /// The number of body in this system.
    pub num_bodies: u8,
}

pub struct Systems {
    /// Since this vec should never be modified at runtime, index are SystemId.
    /// Systems can change from version to version however.
    /// TODO: Add a way to update SystemId to a newer version.
    /// 
    /// Systems are sorted on the y axis from the top.
    pub systems: Vec<System>,
    pub bodies: Vec<CelestialBody>,
    /// Some infos like name and looks that do not affect the simulation.
    pub infos: Vec<CelestialBodyInfo>,
}
impl Systems {
    pub fn get_bound(&self) -> f32 {
        self.systems.iter().fold(0.0f32, |acc, system| {
            acc.max(system.position.length() + system.bound)
        })
    }

    /// Return the position of all bodies in a system in world space.
    /// 
    /// Result should be empty.
    pub fn get_bodies_position(&self, system_index: usize, time: f32, result: &mut Vec<Vec2>) {
        let system = &self.systems[system_index];
        result.reserve(system.num_bodies as usize);
        let first_body_index = system.first_body as usize;
        let last_body_index = system.first_body as usize + system.num_bodies as usize;
        for body in self.bodies[first_body_index..last_body_index].iter() {
            let mut body_pos = body.get_body_relative_position(time) + system.position;
            if let Some(other_offset) = body.parent {
                body_pos += result[other_offset as usize];
            }
            result.push(body_pos);
        }
    }

    /// Return the postion of a single body relative to its parent in world space.
    /// 
    /// This can be used to efficiently calculate the body's position if you know it has no parent.
    pub fn get_body_position(&self, time: f32, system_index: usize, body_offset: usize) -> Vec2 {
        let system = &self.systems[system_index];
        let body = &self.bodies[system.first_body as usize + body_offset];
        system.position + body.get_body_relative_position(time)
    }
}
impl Default for Systems {
    fn default() -> Self {
        Self {
            systems: Vec::new(),
            bodies: Vec::new(),
            infos: Vec::new(),
        }
    }
}
