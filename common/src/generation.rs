use crate::collision::{Collider, Membership};
use crate::metascape::*;
use glam::Vec2;
use rand::{random, Rng};
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use std::f32::consts::PI;

pub struct GenerationMask {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<f32>,
    /// Multiply the value form the buffer.
    pub multiplier: f32,
}
impl GenerationMask {
    /// Sample system density buffer.
    pub fn sample(&self, uv: Vec2) -> f32 {
        let x = (uv.x * self.width as f32) as usize;
        let y = (uv.y * self.height as f32) as usize;
        *self.buffer.get(x + y * self.width).unwrap_or(&0.0)
    }
}

pub struct GenerationParameters {
    pub seed: u64,
    rng: Xoshiro256PlusPlus,

    // TODO: Mods
    pub mods: (),

    /// Sampled to determine the odds of making systems in a sector.
    system_generation_mask: GenerationMask,
}
impl GenerationParameters {
    pub fn new(mut seed: u64, system_generation_mask: GenerationMask) -> Self {
        if seed == 0 {
            seed = random();
        }

        Self {
            seed,
            rng: Xoshiro256PlusPlus::seed_from_u64(seed),
            mods: (),
            system_generation_mask,
        }
    }
}

impl Metascape {
    pub fn generate_system(&mut self, generation_parameters: &mut GenerationParameters) {
        // How many systems we will try to place randomly.
        let num_attempt = ((self.bound.powi(2) * PI) / (System::SIZE.powi(2) * PI)
            * generation_parameters.system_generation_mask.multiplier) as usize;
        debug!("Num system attempt: {}.", num_attempt);

        for attempt_number in 0..num_attempt {
            let completion = attempt_number as f32 / num_attempt as f32;

            let uv: Vec2 = generation_parameters.rng.gen::<Vec2>();
            let position: Vec2 = uv * 2.0 * self.bound - self.bound;

            // Check if we are within metascape bound.
            if position.length_squared() > self.bound.powi(2) {
                continue;
            }

            // Check density.
            if completion > generation_parameters.system_generation_mask.sample(uv) {
                continue;
            }

            // TODO: Temporary size constant. This should come from what is inside the system.
            let radius = System::SIZE;

            // Create system Collider.
            let collider = Collider { radius, position };

            // Test if it overlap with any existing system.
            if self.intersection_pipeline.test_collider(collider, Membership::System) {
                continue;
            }

            // Add this new system.
            let collider_id = self.intersection_pipeline.insert_collider(collider, Membership::System);
            self.systems.insert(collider_id, System {});
            // TODO: Try to add system far apart so we don't have to update every time.
            self.intersection_pipeline.update();
        }

        debug!("Num system inserted: {}.", self.systems.len());

        // TODO: Find neighboring systems.
    }
}
