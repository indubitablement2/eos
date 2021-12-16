use crate::collider::Collider;
use crate::generation::GenerationParameters;
use crate::parameters::MetascapeParameters;
use glam::Vec2;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

pub enum CelestialBodyType {
    /// Used as a center body for system with multiple stars.
    Nothing,
    Star,
    Planet,
}

pub struct CelestialBody {
    pub celestial_body_type: CelestialBodyType,
    pub radius: f32,
    pub orbit_radius: f32,
    pub orbit_time: u64,
    pub moons: Vec<CelestialBody>,
}

pub struct System {
    /// Edge of the outtermost body.
    pub radius: f32,
    pub position: Vec2,
    pub center_body: CelestialBody,
}
impl System {
    pub const RADIUS_MIN: f32 = 32.0;
    pub const RADIUS_MAX: f32 = 128.0;
    const ORBIT_TIME_MIN_PER_RADIUS: u64 = 600;

    /// Return a randomly generated System with its radius.
    /// The ColliderId provided is invalid and needs to be replaced.
    fn generate_system(position: Vec2, size_multiplier: f32, rng: &mut Xoshiro256PlusPlus) -> Self {
        // Get system radius.
        let system_radius = (rng.gen_range((Self::RADIUS_MIN / 0.8)..(Self::RADIUS_MAX * 0.8)) * size_multiplier)
            .clamp(System::RADIUS_MIN, System::RADIUS_MAX);

        // Create System center body.
        let mut center_body = CelestialBody {
            celestial_body_type: CelestialBodyType::Star,
            radius: 8.0,
            orbit_radius: 0.0,
            orbit_time: 0,
            moons: Vec::new(),
        };

        // Add bodies.
        let mut used_radius = center_body.radius;
        while used_radius < system_radius {
            let radius = 1.0;
            let orbit_radius = radius + used_radius + rng.gen_range(1.0..10.0);
            let orbit_time = Self::ORBIT_TIME_MIN_PER_RADIUS * orbit_radius as u64;

            let new_body = CelestialBody {
                celestial_body_type: CelestialBodyType::Planet,
                radius,
                orbit_radius,
                orbit_time,
                moons: Vec::new(),
            };

            center_body.moons.push(new_body);
            used_radius += orbit_radius + radius
        }

        Self {
            radius: used_radius,
            position,
            center_body,
        }
    }
}

/// Since this vec should never be modified, index can be used as id.
/// Systems are sorted on the x axis.
pub struct Systems(pub Vec<System>);
impl Systems {
    const SYSTEM_SAFE_DISTANCE: f32 = 32.0;

    pub fn generate(generation_parameters: &GenerationParameters, metascape_parameters: &MetascapeParameters) -> Self {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(generation_parameters.seed);

        let mut systems = Vec::new();
        let mut system_colliders = Vec::new();

        let bound = metascape_parameters.bound;

        // How many systems we will try to place randomly.
        let num_attempt = (bound.powi(2) / System::RADIUS_MAX.powi(2)) as usize;
        debug!("Num system generation attempt: {}.", num_attempt);

        for attempt_number in 0..num_attempt {
            let completion = attempt_number as f32 / num_attempt as f32;
            let uv: Vec2 = rng.gen::<Vec2>();

            // Check if we are within metascape bound.
            let position: Vec2 = uv * 2.0 * bound - bound;
            if position.length_squared() > bound.powi(2) {
                continue;
            }

            // Check density.
            if completion > generation_parameters.system_density.sample(uv) {
                continue;
            }

            // Generate a random system.
            let system_size_multiplier = generation_parameters.system_size.sample(uv);
            let system = System::generate_system(position, system_size_multiplier, &mut rng);

            // Check if system bound is within metascape bound.
            let system_bound_squared = position.length_squared() + system.radius.powi(2);
            if system_bound_squared > bound.powi(2) {
                continue;
            }

            let collider = Collider::new_idless(system.radius + Self::SYSTEM_SAFE_DISTANCE, position);

            // Test if it overlap with any existing system.
            for other_collider in system_colliders.iter() {
                if collider.intersection_test(*other_collider) {
                    continue;
                }
            }

            // Add system.
            system_colliders.push(collider);
            systems.push(system);
        }

        debug!("Num system generated: {}.", systems.len());

        // Sort systems on the x axis.
        systems.sort_unstable_by(|a, b| {
            a.position
                .x
                .partial_cmp(&b.position.x)
                .expect("this should be a normal float.")
        });

        Self(systems)
    }
}
