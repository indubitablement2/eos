use common::system::*;
use gdnative::prelude::godot_print;
use glam::Vec2;
use rand::prelude::*;
use std::{f32::consts::TAU, ops::Range};

/// 6 sec for a full rotation if 1 time unit == 0.1 sec.
const DEFAULT_ORBIT_SPEED: f32 = 1.0 / (60.0 * TAU);
/// Maximum number of planet sharing the same orbit.
const MAX_PLANET_SHARED_ORBIT: i32 = 3;
/// Extra empty padding added to a system radius.
const SYSTEM_PADDING: Range<f32> = 15.0..25.0;

/// Return a randomly generated System.
pub fn generate_system(position: Vec2, target_radius: f32) -> System {
    let mut rng = thread_rng();

    // Create star.
    let star = if rng.gen_bool(1.0 / 32.0) {
        Star {
            star_type: StarType::BlackHole,
            radius: rng.gen_range(5.0..7.0),
        }
    } else if rng.gen_bool(1.0 / 32.0) {
        Star {
            star_type: StarType::Nebula,
            radius: 0.0,
        }
    } else {
        Star {
            star_type: StarType::Star,
            radius: rng.gen_range(6.0..12.0),
        }
    };

    let mut used_radius = star.radius;
    let mut planets = Vec::new();

    // Add planets.
    while used_radius < target_radius || planets.len() < 2 {
        let radius = rng.gen_range(1.0..2.0);
        let distance = radius + used_radius + rng.gen_range(6.0..12.0);
        let orbit_speed = DEFAULT_ORBIT_SPEED / distance
            * rng.gen_range(0.5..2.0)
            * (rng.gen::<f32>() - 0.5).signum();
        let start_angle_rand = rng.gen::<f32>() * TAU;

        let num_planet: i32 = rng.gen_range(1..MAX_PLANET_SHARED_ORBIT);
        for i in 0..num_planet {
            planets.push(Planet {
                radius,
                relative_orbit: common::orbit::RelativeOrbit {
                    distance,
                    start_angle: TAU * i as f32 / num_planet as f32 + start_angle_rand,
                    orbit_speed,
                },
            });

            used_radius = radius.mul_add(2.0, distance).max(used_radius);
        }
    }

    let system = System {
        radius: used_radius + rng.gen_range(SYSTEM_PADDING),
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
