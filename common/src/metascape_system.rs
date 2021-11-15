pub enum CelestialBodyType {
    Star,
    Planet,
}

pub struct CelestialBody {
    pub celestial_body_type: CelestialBodyType,
    pub radius: f32,
    pub orbit_radius: f32,
    /// How many timestep for a full rotation.
    pub orbit_time: u32,
    pub moons: Vec<CelestialBody>,
}

/// A system with stars and planets.
pub struct System {
    /// The body that is the center of this system. Usualy a single star.
    pub bodies: Vec<CelestialBody>,
}
impl System {
    pub const RADIUS_MIN: f32 = 64.0;
    pub const RADIUS_MAX: f32 = 256.0;
    /// Final System radius is added a bound with nothing in it.
    pub const BOUND_RADIUS_MULTIPLER: f32 = 1.25;
    // pub const BODIES_MAX: u32 = 32;
    /// Miminum number of timestep for a full rotation for every 1.0 away from main body.
    pub const ORBIT_TIME_MIN_PER_RADIUS: u32 = 600;
}
