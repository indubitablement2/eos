use common::collider::Collider;
use common::system::*;
use glam::Vec2;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

const ORBIT_TIME_MIN_PER_RADIUS: u32 = 600;

/// Return a randomly generated System with its radius.
/// The ColliderId provided is invalid and needs to be replaced.
fn generate_system(position: Vec2, max_radius: f32, rng: &mut Xoshiro256PlusPlus) -> System {
    let mut bodies = Vec::new();

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
    while used_radius < max_radius {
        let radius = 1.0;
        let orbit_radius = radius + used_radius + rng.gen_range(1.0..10.0);
        let orbit_time = ORBIT_TIME_MIN_PER_RADIUS * orbit_radius as u32;

        let new_used_radius = used_radius + orbit_radius + radius;
        if new_used_radius > max_radius {
            break;
        }

        used_radius = new_used_radius;
        bodies.push(CelestialBody {
            body_type: CelestialBodyType::Planet,
            radius,
            parent: CelestialBodyParent::CelestialBody(0),
            orbit_radius,
            orbit_time,
        });
    }

    System {
        seed: rng.gen(),
        radius: used_radius,
        position,
        bodies,
    }
}

pub fn generate_systems(
    seed: u64,
    bound: f32,
    radius_min: f32,
    radius_max: f32,
    min_distance: f32,
    system_density: f32,
    system_size: f32,
) -> Systems {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);

    let mut systems = Vec::new();
    let mut colliders = Vec::new();

    let bound_squared = bound.powi(2);

    // How many systems we will try to place randomly.
    let num_attempt = (bound_squared * system_density / radius_max.powi(2)) as usize;
    'outer: for _ in 0..num_attempt {
        // Check if we are within metascape bound.
        let position: Vec2 = rng.gen::<Vec2>() * 2.0 * bound - bound;
        if position.length_squared() > bound_squared {
            continue;
        }

        // How big we will try to make this system.
        let max_system_radius =
            (rng.gen_range((radius_min / 0.8)..(radius_max * 0.8)) * system_size).clamp(radius_min, radius_max);

        // Generate a random system.
        let system = generate_system(position, max_system_radius, &mut rng);

        // Check if system bound is within metascape bound.
        let system_bound_squared = (system.position.length() + system.radius).powi(2);
        if system_bound_squared > bound_squared {
            continue;
        }

        let collider = Collider::new_idless(system.radius + min_distance, position);

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
    let success_rate = ((systems.len() as f32 / num_attempt as f32) * 100.0) as i32;
    debug!("Systems generated: {}/{}  {}%.", systems.len(), num_attempt, success_rate);

    // Sort systems on the x axis.
    systems.sort_unstable_by(|a, b| {
        a.position
            .x
            .partial_cmp(&b.position.x)
            .expect("this should be a real number.")
    });

    Systems(systems)
}
