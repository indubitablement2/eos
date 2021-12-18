use std::f32::consts::TAU;

use glam::Vec2;

pub enum CelestialBodyType {
    Star,
    Planet,
}

/// Represent either a position or another `CelestialBody`.
pub enum CelestialBodyParent {
    /// This `CelestialBody` is orbiting another `CelestialBody`.
    CelestialBody(usize),
    /// This `CelestialBody` is orbiting a static position.
    StaticPosition(Vec2)
}

pub struct CelestialBody {
    pub body_type: CelestialBodyType,
    pub radius: f32,

    /// What is this `CelestialBody` orbiting around.
    pub parent: CelestialBodyParent,
    /// How far is this `CelestialBody` orbiting from its origin.
    pub orbit_radius: f32,
    /// How many tick does it take this `CelestialBody` to complete an orbit.
    pub orbit_time: f32,
}
impl CelestialBody {
    /// Return the orbit rotation in radian.
    pub fn get_orbit_rotation(&self, tick: f32) -> f32 {
        (tick / self.orbit_time) * TAU
    }
}

pub struct System {
    /// Number used to derive some deterministic values.
    pub seed: u64,

    /// Edge of the outtermost `CelestialBody`.
    pub radius: f32,
    /// The center of this `System`
    pub position: Vec2,

    /// Bodies are ordered by parent -> child.
    /// The first body can not be a child.
    pub bodies: Vec<CelestialBody>,
}
impl System {
    /// Return the position of bodies relative to system center.
    pub fn get_bodies_system_position(&self, tick: f32) -> Vec<Vec2> {
        let mut positions = Vec::with_capacity(self.bodies.len());
        for body in self.bodies.iter() {
            let origin = match body.parent {
                CelestialBodyParent::CelestialBody(other) => positions[other],
                CelestialBodyParent::StaticPosition(p) => p,
            };
            let rot = body.get_orbit_rotation(tick);
            positions.push(Vec2::new(f32::cos(rot), f32::sin(rot)) * body.orbit_radius + origin);
        }
        positions
    }

    /// Return the position of bodies relative to world center.
    pub fn get_bodies_world_position(&self, tick: f32) -> Vec<Vec2> {
        let mut positions = self.get_bodies_system_position(tick);
        for pos in positions.iter_mut() {
            *pos += self.position;
        }
        positions
    }
}

/// Since this vec should never be modified, index can be used as id.
/// Index can change from version to version however.
/// TODO: Add a way to update SystemId to a newer version.
/// 
/// Systems are sorted on the x axis.
pub struct Systems(pub Vec<System>);
impl Default for Systems {
    fn default() -> Self {
        Self(Vec::new())
    }
}
