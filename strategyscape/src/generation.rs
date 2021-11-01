use crate::*;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rapier2d::na::{Isometry2, Vector2};

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

    pub fn generate_system(&mut self, strategyscape: &mut Strategyscape) {
        // How many systems we will try to place randomly.
        let num_attempt =
            (strategyscape.bound.volume() / (System::SMALL * 2.0).powi(2) * self.system_density_multiplier) as usize;

        let translation: Vector2<f32> = vector![
            self.rng.gen_range(strategyscape.bound.mins.x..strategyscape.bound.maxs.x),
            self.rng.gen_range(strategyscape.bound.mins.y..strategyscape.bound.maxs.y)
        ];
        // TODO: Divide in quadrant.

        for attempt_number in 0..num_attempt {
            let completion = attempt_number as f32 / num_attempt as f32;

            let translation: Vector2<f32> = vector![
                self.rng.gen_range(strategyscape.bound.mins.x..strategyscape.bound.maxs.x),
                self.rng.gen_range(strategyscape.bound.mins.y..strategyscape.bound.maxs.y)
            ];

            let uv: Vector2<f32> = (translation + strategyscape.bound.half_extents()).component_div(&strategyscape.bound.extents());

            // Check density.
            if completion > self.sample_system_density(uv) {
                continue;
            }

            let radius = match self.rng.gen_range(0..3) {
                0 => System::SMALL,
                1 => System::MEDIUM,
                _ => System::LARGE,
            };

            let collider = ColliderBuilder::ball(radius)
                .sensor(true)
                .active_events(ActiveEvents::INTERSECTION_EVENTS)
                .translation(translation)
                .collision_groups(InteractionGroups::new(
                    System::COLLISION_MEMBERSHIP,
                    Player::COLLISION_MEMBERSHIP,
                ))
                .build();

            // Test if it overlap with any existing system.
            if strategyscape
                .query_pipeline_bundle
                .query_pipeline
                .intersection_with_shape(
                    &strategyscape.body_set_bundle.collider_set,
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
            let collider_handle = strategyscape.body_set_bundle.collider_set.insert(collider);
            strategyscape.systems.insert(collider_handle, System {});
            // strategyscape.query_pipeline_bundle.update(&strategyscape.body_set_bundle);
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
