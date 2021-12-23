use common::intersection::Collider;
use common::system::*;
use glam::Vec2;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use std::convert::TryFrom;

const ORBIT_TIME_MIN_PER_RADIUS: f32 = 600.0;

// /// Prevalence: 0.01,
// /// Radius: 2.5..10
// BlueStar,
// /// Prevalence: 0.01,
// /// Radius: 0.95..1,5
// YellowDwarf,
// /// Prevalence: 0.01,
// /// Radius: 0.7..0.95
// OrangeDwarf,
// /// Prevalence: 0.73,
// /// Radius: 0.6..0.8
// RedDwarf,
// /// Prevalence: 0.01,
// /// Radius: 5..10
// BlueGiant,
// /// Prevalence: 0.01,
// /// Radius: 18..25
// BlueSuperGiant,
// /// Prevalence: 0.01,
// /// Radius: 20..100
// RedGiant,
// /// Prevalence: 0.01,
// /// Radius: 100..1700
// RedSuperGiant,
// /// Prevalence: 0.4,
// /// Radius: 0.1..0.2
// WhiteDwarf,
// /// Prevalence: 0.1,
// /// Radius: 5..15
// NeutronStar,
// /// Prevalence: 0.001,
// /// Radius: 0.1..0.2
// BlackDwarf,
// BlackHole,
// /// Prevalence: 0.1,
// /// Radius: 0.05..0.1
// BrownDwarf,

// Planet,
// GasGiant,
// Moon,

/// Return a randomly generated System with its radius.
/// The ColliderId provided is invalid and needs to be replaced.
fn generate_system(position: Vec2, max_radius: f32, rng: &mut Xoshiro256PlusPlus) -> (System, Vec<CelestialBody>) {
    let mut bodies = Vec::new();

    // Create System center body.
    let center_body = CelestialBody {
        body_type: CelestialBodyType::Star,
        radius: 8.0,
        parent: None,
        orbit_radius: 0.0,
        orbit_time: 1.0,
    };
    let mut used_radius = center_body.radius;
    bodies.push(center_body);

    // Add bodies.
    loop {
        let radius = rng.gen_range(0.4..4.0);
        let orbit_radius = radius + used_radius + rng.gen_range(1.0..10.0);
        let orbit_time = ORBIT_TIME_MIN_PER_RADIUS * orbit_radius * (rng.gen::<f32>() - 0.5).signum();

        let new_used_radius = orbit_radius + radius;
        if new_used_radius > max_radius {
            break;
        }

        used_radius = new_used_radius;
        bodies.push(CelestialBody {
            body_type: CelestialBodyType::Planet,
            radius,
            parent: None,
            orbit_radius,
            orbit_time,
        });
    }

    let system = System {
        bound: used_radius,
        position,
        first_body: 0,
        num_bodies: u8::try_from(bodies.len()).unwrap(),
    };

    (system, bodies)
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
    let mut bodies = Vec::new();
    let mut colliders = Vec::new();

    let systems_collider = Collider::new_idless(bound, Vec2::ZERO);

    // How many systems we will try to place randomly.
    let num_attempt = (bound.powi(2) * system_density / radius_max.powi(2)) as usize;
    'outer: for _ in 0..num_attempt {
        // Check if we are within metascape bound.
        let position: Vec2 = rng.gen::<Vec2>() * 2.0 * bound - bound;
        if !systems_collider.intersection_test_point(position) {
            continue;
        }

        // How big we will try to make this system.
        let max_system_radius =
            (rng.gen_range((radius_min / 0.8)..(radius_max * 0.8)) * system_size).clamp(radius_min, radius_max);

        // Generate a random system.
        let (mut system, system_bodies) = generate_system(position, max_system_radius, &mut rng);

        let system_collider = Collider::new_idless(system.bound, system.position);

        // Check if system bound is within metascape bound.
        if !systems_collider.incorporate_test(system_collider) {
            continue;
        }

        let system_collider_safe = Collider::new_idless(system.bound + min_distance, system.position);

        // Test if it overlap with any existing system.
        for other_collider in colliders.iter() {
            if system_collider_safe.intersection_test(*other_collider) {
                continue 'outer;
            }
        }

        // Add system.
        colliders.push(system_collider);
        system.first_body = bodies.len() as u32;
        systems.push(system);
        bodies.extend(system_bodies.into_iter());
    }
    let success_rate = ((systems.len() as f32 / num_attempt as f32) * 100.0) as i32;
    debug!(
        "Systems generated: {}/{}  {}%.",
        systems.len(),
        num_attempt,
        success_rate
    );

    // Sort systems on the y axis.
    systems.sort_unstable_by(|a, b| {
        a.position
            .y
            .partial_cmp(&b.position.x)
            .expect("this should be a real number.")
    });

    Systems {
        systems,
        bodies,
        infos: Vec::new(),
    }
}
