use common::systems::*;
use gdnative::prelude::godot_print;
use glam::Vec2;
use rand::Rng;
use std::f32::consts::TAU;

/// 6 sec for a full rotation if 1 time unit == 0.1 sec.
const DEFAULT_ORBIT_SPEED: f32 = 1.0 / (60.0 * TAU);

/// Return a randomly generated System with its radius.
pub fn generate_system(position: Vec2, target_radius: f32) -> System {
    let mut rng = rand::thread_rng();

    // Create central body.
    let star = if rng.gen_bool(1.0 / 32.0) {
        Star {
            star_type: StarType::BlackHole,
            radius: rng.gen_range(5.0..7.0),
            temperature: 0.0,
        }
    } else if rng.gen_bool(1.0 / 32.0) {
        Star {
            star_type: StarType::Nebula,
            radius: 0.0,
            temperature: 0.1,
        }
    } else {
        Star {
            star_type: StarType::Star,
            radius: rng.gen_range(6.0..12.0),
            temperature: rng.gen_range(0.1..1.0),
        }
    };

    let mut used_radius = star.radius;
    let mut planets = Vec::new();

    // Add planets.
    while used_radius < target_radius || planets.len() < 2 {
        let radius = rng.gen_range(1.0..2.0);
        let distance = radius + used_radius + rng.gen_range(6.0..12.0);
        let orbit_speed = DEFAULT_ORBIT_SPEED / distance * rng.gen_range(0.5..2.0) * (rng.gen::<f32>() - 0.5).signum();
        let start_angle_rand = rng.gen::<f32>() * TAU;

        let num_planet: i32 = rng.gen_range(1..3);
        for i in 0..num_planet {
            planets.push(Planet {
                radius,
                relative_orbit: common::orbit::RelativeOrbit {
                    distance,
                    start_angle: TAU * i as f32 / num_planet as f32 + start_angle_rand,
                    orbit_speed,
                },
                temperature: 0.0,
                faction: None,
                population: 0,
            });

            used_radius = radius.mul_add(2.0, distance).max(used_radius);
        }
    }

    let system = System {
        bound: used_radius + System::PADDING,
        position,
        star,
        planets,
    };

    godot_print!(
        "Generated {} system with {} planets.",
        system.star.star_type.to_str(),
        system.planets.len()
    );

    system
}
