use common::collider::Collider;
use common::parameters::MetascapeParameters;
use common::system::*;
use glam::Vec2;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::generation_mask::GenerationParameters;

const RADIUS_MIN: f32 = 32.0;
const RADIUS_MAX: f32 = 128.0;
const ORBIT_TIME_MIN_PER_RADIUS: u32 = 600;
const SYSTEM_SAFE_DISTANCE: f32 = 32.0;

/// Return a randomly generated System with its radius.
/// The ColliderId provided is invalid and needs to be replaced.
fn generate_system(position: Vec2, size_multiplier: f32, rng: &mut Xoshiro256PlusPlus) -> System {
    let mut bodies = Vec::new();

    // How big we will try to make this system.
    let target_system_radius = (rng.gen_range((RADIUS_MIN / 0.8)..(RADIUS_MAX * 0.8)) * size_multiplier)
        .clamp(RADIUS_MIN, RADIUS_MAX);

    // Create System center body.
    let center_body = CelestialBody {
        body_type: CelestialBodyType::Star,
        radius: 8.0,
        parent: CelestialBodyParent::StaticPosition(Vec2::ZERO),
        orbit_radius: 0.0,
        orbit_time: 0,
    };
    let mut used_radius = center_body.radius;
    bodies.push(center_body);

    // Add bodies.
    while used_radius < target_system_radius {
        let radius = 1.0;
        let orbit_radius = radius + used_radius + rng.gen_range(1.0..10.0);
        let orbit_time = ORBIT_TIME_MIN_PER_RADIUS * orbit_radius as u32;

        bodies.push( CelestialBody {
            body_type: CelestialBodyType::Planet,
            radius,
            parent: CelestialBodyParent::CelestialBody(0),
            orbit_radius,
            orbit_time,
        });
        used_radius += orbit_radius + radius
    }

    System {
        seed: rng.gen(),
        radius: used_radius,
        position,
        bodies,
    }
}

pub fn generate_systems(generation_parameters: &GenerationParameters) -> Systems {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(generation_parameters.seed);

    let mut systems = Vec::new();
    let mut colliders = Vec::new();

    let bound = generation_parameters.bound;

    // How many systems we will try to place randomly.
    let num_attempt = (bound.powi(2) / RADIUS_MAX.powi(2)) as usize;
    debug!("Num system generation attempt: {}.", num_attempt);

    'outer: for attempt_number in 0..num_attempt {
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
        let system = generate_system(position, system_size_multiplier, &mut rng);

        // Check if system bound is within metascape bound.
        let system_bound_squared = position.length_squared() + system.radius.powi(2);
        if system_bound_squared > bound.powi(2) {
            continue;
        }

        let collider = Collider::new_idless(system.radius + SYSTEM_SAFE_DISTANCE, position);

        // Test if it overlap with any existing system.
        for other_collider in colliders.iter() {
            if collider.intersection_test(*other_collider) {
                continue 'outer;
            }
        }

        // Add system.
        colliders.push(collider);
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

    Systems(systems)
}
