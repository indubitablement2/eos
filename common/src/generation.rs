use crate::collision::{Collider, Membership};
use crate::metascape::*;
use crate::metascape_system::*;
use glam::Vec2;
use rand::{random, Rng};
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

pub struct GenerationMask {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<f32>,
    /// Multiply the value form the buffer.
    pub multiplier: f32,
}
impl GenerationMask {
    /// Try to sample the buffer at the given position or return 1.0.
    pub fn sample(&self, uv: Vec2) -> f32 {
        let x = (uv.x * self.width as f32) as usize;
        let y = (uv.y * self.height as f32) as usize;
        *self.buffer.get(x + y * self.width).unwrap_or(&1.0) * self.multiplier
    }
}
impl Default for GenerationMask {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            buffer: vec![],
            multiplier: 1.0,
        }
    }
}

pub struct GenerationParameters {
    pub seed: u64,
    rng: Xoshiro256PlusPlus,

    // TODO: Mods
    pub mods: (),

    /// Sampled to determine the odds of making systems in a sector.
    system_density: GenerationMask,
    system_size: GenerationMask,
}
impl GenerationParameters {
    pub fn new(mut seed: u64, system_density: GenerationMask, system_size: GenerationMask) -> Self {
        if seed == 0 {
            seed = random();
        }

        Self {
            seed,
            rng: Xoshiro256PlusPlus::seed_from_u64(seed),
            mods: (),
            system_density,
            system_size,
        }
    }

    /// Return a randomly generated System with its radius.
    fn generate_system(&mut self, uv: Vec2) -> (System, f32) {
        // Get system radius.
        let system_radius = (self.rng.gen_range((System::RADIUS_MIN / 0.8)..(System::RADIUS_MAX * 0.8))
            * self.system_size.sample(uv))
        .clamp(System::RADIUS_MIN, System::RADIUS_MAX);

        // Create System center body.
        let mut main_body = CelestialBody {
            celestial_body_type: CelestialBodyType::Star,
            radius: 8.0,
            orbit_radius: 0.0,
            orbit_time: 0,
            moons: Vec::new(),
        };

        // Add bodies.
        let mut used_radius = main_body.radius;
        while used_radius < system_radius {
            let radius = 1.0;
            let orbit_radius = radius + used_radius + self.rng.gen_range(1.0..10.0);
            let orbit_time = System::ORBIT_TIME_MIN_PER_RADIUS * orbit_radius as u32;

            let new_body = CelestialBody {
                celestial_body_type: CelestialBodyType::Planet,
                radius,
                orbit_radius,
                orbit_time,
                moons: Vec::new(),
            };

            main_body.moons.push(new_body);
            used_radius += orbit_radius + radius
        }

        (
            System { bodies: vec![main_body] },
            system_radius * System::BOUND_RADIUS_MULTIPLER,
        )
    }
}

impl Metascape {
    pub fn generate(&mut self, generation_parameters: &mut GenerationParameters) {
        // How many systems we will try to place randomly.
        let num_attempt = (self.bound.powi(2) / System::RADIUS_MAX.powi(2)) as usize;
        debug!("Num system attempt: {}.", num_attempt);

        for attempt_number in 0..num_attempt {
            let completion = attempt_number as f32 / num_attempt as f32;
            let uv: Vec2 = generation_parameters.rng.gen::<Vec2>();

            // Check if we are within metascape bound.
            let position: Vec2 = uv * 2.0 * self.bound - self.bound;
            if position.length_squared() > self.bound.powi(2) {
                continue;
            }

            // Check density.
            if completion > generation_parameters.system_density.sample(uv) {
                continue;
            }

            // Generate a random system.
            let (system, radius) = generation_parameters.generate_system(uv);

            // Create system Collider.
            let collider = Collider { radius, position };

            // Test if it overlap with any existing system.
            if self.intersection_pipeline.test_collider(collider, Membership::System) {
                continue;
            }

            // Add this new system.
            let collider_id = self.intersection_pipeline.insert_collider(collider, Membership::System);
            self.systems.insert(collider_id, system);
            // TODO: Try to add system far apart so we don't have to update every time.
            self.intersection_pipeline.update();
        }

        debug!("Num system inserted: {}.", self.systems.len());

        // TODO: Find neighboring systems.
    }
}
