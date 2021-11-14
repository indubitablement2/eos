use crate::collision::{Collider, Membership};
use crate::metascape::*;
use glam::Vec2;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use std::f32::consts::PI;

pub struct GenerationParameters {
    pub seed: u64,
    pub rng: Xoshiro256PlusPlus,

    // TODO: Mods
    pub mods: (),

    pub system_density_buffer_height: usize,
    pub system_density_buffer_width: usize,
    /// Sampled to determine the odds of making systems in a sector. Usualy comes from an image.
    pub system_density_buffer: Vec<f32>,
    /// Multiply the system_density_buffer.
    pub system_density_multiplier: f32,
}
impl GenerationParameters {
    pub fn get_rgn_from_seed(seed: u64) -> Xoshiro256PlusPlus {
        Xoshiro256PlusPlus::seed_from_u64(seed)
    }

    pub fn generate_system(&mut self, metascape: &mut Metascape) {
        // How many systems we will try to place randomly.
        let num_attempt =
            ((metascape.bound.powi(2) * PI) / (System::SIZE.powi(2) * PI) * self.system_density_multiplier) as usize;
        debug!("Num system attempt: {}.", num_attempt);

        for attempt_number in 0..num_attempt {
            let completion = attempt_number as f32 / num_attempt as f32;

            let uv: Vec2 = self.rng.gen::<Vec2>() * 2.0 - 1.0;
            let position: Vec2 = uv * metascape.bound;

            // Check if we are within metascape bound.
            if position.length_squared() > metascape.bound.powi(2) {
                continue;
            }

            // Check density.
            if completion > self.sample_system_density(uv) {
                continue;
            }

            // TODO: Temporary size constant. This should come from what is inside the system.
            let radius = System::SIZE;

            // Create system Collider.
            let collider = Collider { radius, position };

            // Test if it overlap with any existing system.
            if metascape
                .intersection_pipeline
                .intersect_collider(collider, Membership::System)
            {
                continue;
            }

            // Add this new system.
            let collider_id = metascape.intersection_pipeline.insert_collider(collider, Membership::System);
            metascape.systems.insert(collider_id, System {});
            // TODO: Try to add system far apart so we don't have to update every time.
            metascape.intersection_pipeline.update();
        }

        // TODO: Find neighboring systems.
    }

    /// Sample system density buffer.
    pub fn sample_system_density(&self, uv: Vec2) -> f32 {
        let x = (uv.x * self.system_density_buffer_width as f32) as usize;
        let y = (uv.y * self.system_density_buffer_height as f32) as usize;
        *self
            .system_density_buffer
            .get(x + y * self.system_density_buffer_width)
            .unwrap_or(&0.0)
    }
}
