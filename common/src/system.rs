use glam::Vec2;

pub enum CelestialBodyType {
    Star,
    Planet,
}

/// Represent either a position or another `CelestialBody`.
pub enum CelestialBodyParent {
    /// This `CelestialBody` is orbiting another `CelestialBody`.
    /// 
    /// Although `CelestialBody` within a `System` are tipicaly identified with u8, 
    /// this is an u64 here as it take that same amount of memory.
    CelestialBody(u64),
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
    pub orbit_time: u32,
}

pub struct System {
    /// Number used to derive some deterministic values.
    pub seed: u64,

    /// Edge of the outtermost `CelestialBody`.
    pub radius: f32,
    /// The center of this `System`
    pub position: Vec2,

    pub bodies: Vec<CelestialBody>,
}
impl System {
    pub fn get_bodies_position(&self,) {
        
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
