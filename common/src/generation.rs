use crate::metascape::*;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rapier2d::{
    na::{vector, Isometry2, Vector2},
    prelude::*,
};

pub struct GenerationParameters {
    pub seed: u64,
    pub rng: Xoshiro256PlusPlus,

    // TODO:
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
        let num_attempt = (metascape.bound.volume() / (System::SIZE * 2.0).powi(2) * self.system_density_multiplier) as usize;

        for attempt_number in 0..num_attempt {
            let completion = attempt_number as f32 / num_attempt as f32;

            let translation: Vector2<f32> = vector![
                self.rng.gen_range(metascape.bound.mins.x..metascape.bound.maxs.x),
                self.rng.gen_range(metascape.bound.mins.y..metascape.bound.maxs.y)
            ];

            let uv: Vector2<f32> = (translation + metascape.bound.half_extents()).component_div(&metascape.bound.extents());

            // Check density.
            if completion > self.sample_system_density(uv) {
                continue;
            }

            // TODO: Temporary size constant. This should come from what is inside the system.
            let radius = System::SIZE;

            let collider = ColliderBuilder::ball(radius)
                .sensor(true)
                .active_events(ActiveEvents::INTERSECTION_EVENTS)
                .translation(translation)
                .collision_groups(InteractionGroups::new(System::COLLISION_MEMBERSHIP, 0))
                .build();

            // Test if it overlap with any existing system.
            if metascape
                .query_pipeline_bundle
                .query_pipeline
                .intersection_with_shape(
                    &metascape.collider_set,
                    &Isometry2::new(translation, 0.0),
                    collider.shape(),
                    InteractionGroups::all(),
                    None,
                )
                .is_some()
            {
                continue;
            }

            // Add this circle as a new system.
            let collider_handle = metascape.collider_set.insert(collider);
            metascape.systems.insert(collider_handle, System {});
            metascape.query_pipeline_bundle.update(&metascape.collider_set);
        }

        // TODO: Find neighboring systems.
    }

    /// Sample system density buffer.
    pub fn sample_system_density(&self, uv: Vector2<f32>) -> f32 {
        let x = (uv.x * self.system_density_buffer_width as f32) as usize;
        let y = (uv.y * self.system_density_buffer_height as f32) as usize;
        *self
            .system_density_buffer
            .get(x + y * self.system_density_buffer_width)
            .unwrap_or(&0.0)
    }
}
