use common::orbit::Orbit;
use common::world_data::*;
use glam::Vec2;
use rand::Rng;
use std::f32::consts::TAU;

/// Return a randomly generated System with its radius.
pub fn generate_system(position: Vec2, target_radius: f32) -> System {
    let mut rng = rand::thread_rng();

    let mut bodies = Vec::new();

    // Determine how many central body we will add.
    let num_star = if rng.gen_bool(1.0 / 64.0) {
        // No central body.
        0
    } else {
        let mut num = 1;
        while rng.gen_bool(target_radius.min(1000.0) as f64 / 4000.0) {
            // Multiple central body.
            num += 1;
        }
        num
    };

    // Determine if we will place star or black hole as central body.
    let bh = rng.gen_bool(1.0 / 128.0);

    // Create central body.
    let distance = if num_star > 1 {
        64.0 * num_star as f32 / TAU
    } else {
        0.0
    };
    let mut orbit = Orbit {
        origin: position,
        distance,
        start_angle: 0.0,
        orbit_speed: if num_star > 1 {
            Orbit::DEFAULT_ORBIT_SPEED / distance
        } else {
            0.0
        },
    };
    for i in 0..num_star {
        orbit.start_angle = TAU * i as f32 / num_star as f32;

        let body = if bh {
            CelestialBody {
                body_type: CelestialBodyType::BlackHole,
                radius: 16.0,
                orbit,
                name: rng.gen::<u32>().to_string(),
                temperature: 0.0,
                faction: None,
                population: 0,
            }
        } else {
            CelestialBody {
                body_type: CelestialBodyType::Star,
                radius: 16.0,
                orbit,
                name: rng.gen::<u32>().to_string(),
                temperature: rng.gen_range(0.1..1.0),
                faction: None,
                population: 0,
            }
        };

        bodies.push(body);
    }

    let mut used_radius = 16.0 + distance;

    // Add bodies.
    while used_radius < target_radius {
        let radius = rng.gen_range(0.8..1.25);
        let distance = radius + used_radius + rng.gen_range(1.0..16.0);
        let orbit_speed =
            Orbit::DEFAULT_ORBIT_SPEED / distance * rng.gen_range(0.5..2.0) * (rng.gen::<f32>() - 0.5).signum();

        bodies.push(CelestialBody {
            body_type: CelestialBodyType::Planet,
            radius: 1.0,
            orbit: Orbit {
                origin: position,
                distance,
                start_angle: rng.gen::<f32>() * TAU,
                orbit_speed,
            },
            name: rng.gen::<u32>().to_string(),
            temperature: 0.0,
            faction: None,
            population: 0,
        });

        used_radius = radius.mul_add(2.0, distance);
    }

    let mut system = System {
        bound: used_radius + System::PADDING,
        position,
        bodies,
    };

    system.compute_temperature();

    system
}
