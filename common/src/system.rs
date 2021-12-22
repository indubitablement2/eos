use std::f32::consts::TAU;
use glam::Vec2;

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
    /// If this body is orbiting around another body (like a moon around its planet or a planet around a start),
    /// this is the index of the other body.
    /// 
    /// Otherwise it is orbiting around the system center.
    pub parent: Option<u8>,
    /// The distance it is orbiting from its parent.
    pub orbit_radius: f32,
    /// How many tick does it take this `CelestialBody` to complete an orbit.
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

    /// Bodies are ordered by parent -> child.
    /// The first body can not be a child.
    pub bodies: Vec<CelestialBody>,
}
impl System {
    /// Return the position of bodies relative to the system center.
    pub fn get_bodies_system_position(&self, time: f32) -> Vec<Vec2> {
        let mut positions = Vec::with_capacity(self.bodies.len());
        for body in self.bodies.iter() {
            let mut body_position = body.get_body_relative_position(time);
            if let Some(other) = body.parent {
                body_position += positions[usize::from(other)];
            }
            positions.push(body_position);
        }
        positions
    }

    /// Return the position of bodies relative to world center.
    pub fn get_bodies_world_position(&self, time: f32) -> Vec<Vec2> {
        let mut positions = self.get_bodies_system_position(time);
        for pos in positions.iter_mut() {
            *pos += self.position;
        }
        positions
    }

    /// Return the postion relative to its parent of a single body.
    /// 
    /// This can be used to efficiently calculate the body's position if you know it has no parent.
    pub fn get_body_system_position(&self, time: f32, body_id: u8) -> Vec2 {
        self.bodies[usize::from(body_id)].get_body_relative_position(time)
    }
}

/// Since this vec should never be modified at runtime, index can be used as id.
/// Index can change from version to version however.
/// TODO: Add a way to update SystemId to a newer version.
/// 
/// Systems are sorted on the y axis.
pub struct Systems {
    pub systems: Vec<System>,
    /// Edge of the outtermost system's `CelestialBody`.
    pub bound: f32,
}
impl Default for Systems {
    fn default() -> Self {
        Self { systems: Vec::new(), bound: 0.0 }
    }
}
