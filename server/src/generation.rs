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
}

//     /// Return a randomly generated System with its radius.
//     /// The ColliderId provided is invalid and needs to be replaced.
//     fn generate_system(&mut self, uv: Vec2, successful_attempt: u16) -> (System, f32) {
//         // Get system radius.
//         let system_radius = (self.rng.gen_range((System::RADIUS_MIN / 0.8)..(System::RADIUS_MAX * 0.8))
//             * self.system_size.sample(uv))
//         .clamp(System::RADIUS_MIN, System::RADIUS_MAX);

//         // Create System center body.
//         let mut main_body = CelestialBody {
//             celestial_body_type: CelestialBodyType::Star,
//             radius: 8.0,
//             orbit_radius: 0.0,
//             orbit_time: 0,
//             moons: Vec::new(),
//         };

//         // Add bodies.
//         let mut used_radius = main_body.radius;
//         while used_radius < system_radius {
//             let radius = 1.0;
//             let orbit_radius = radius + used_radius + self.rng.gen_range(1.0..10.0);
//             let orbit_time = System::ORBIT_TIME_MIN_PER_RADIUS * orbit_radius as u32;

//             let new_body = CelestialBody {
//                 celestial_body_type: CelestialBodyType::Planet,
//                 radius,
//                 orbit_radius,
//                 orbit_time,
//                 moons: Vec::new(),
//             };

//             main_body.moons.push(new_body);
//             used_radius += orbit_radius + radius
//         }

//         (
//             System {
//                 bodies: vec![main_body],
//                 collider_id: ColliderId::new_invalid(),
//             },
//             system_radius * System::BOUND_RADIUS_MULTIPLER,
//         )
//     }

//     pub fn generate(&mut self, metascape_params: &ParametersRes) {
//         let bound = metascape.parameters.bound;

//         // How many systems we will try to place randomly.
//         let num_attempt = (bound.powi(2) / System::RADIUS_MAX.powi(2)) as usize;
//         let mut successful_attempt = 0u16;
//         debug!("Num system attempt: {}.", num_attempt);

//         for attempt_number in 0..num_attempt {
//             let completion = attempt_number as f32 / num_attempt as f32;
//             let uv: Vec2 = self.rng.gen::<Vec2>();

//             // Check if we are within metascape bound.
//             let position: Vec2 = uv * 2.0 * bound - bound;
//             if position.length_squared() > bound.powi(2) {
//                 continue;
//             }

//             // Check density.
//             if completion > self.system_density.sample(uv) {
//                 continue;
//             }

//             // Generate a random system.
//             let (mut system, radius) = self.generate_system(uv, successful_attempt);

//             // Create system Collider.
//             let collider = Collider {
//                 radius,
//                 position,
//                 custom_data: successful_attempt.into(),
//             };

//             // Test if it overlap with any existing system.
//             if metascape.intersection_pipeline.test_collider(collider, Membership::System) {
//                 continue;
//             }

//             // Add collider.
//             let collider_id = metascape.intersection_pipeline.insert_collider(collider, Membership::System);
//             system.collider_id = collider_id;
//             // Add this new system.
//             metascape.systems.insert(SystemId { id: successful_attempt }, system);
//             // TODO: Try to add system far apart so we don't have to update every time.
//             metascape.intersection_pipeline.update();
//             successful_attempt += 1;
//         }

//         debug!("Num system inserted: {}.", successful_attempt);
//     }
// }
