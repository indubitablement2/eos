use common::orbit::Orbit;
use common::world_data::*;
use glam::Vec2;
use rand::Rng;
use std::f32::consts::TAU;

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
pub fn generate_system(position: Vec2, max_radius: f32) -> System {
    let origin = position;
    let mut rng = rand::thread_rng();

    let mut bodies = Vec::new();

    // Create System center body.
    let center_body = CelestialBody {
        body_type: CelestialBodyType::Star,
        radius: rng.gen_range(4.0..16.0),
        orbit: Orbit::stationary(origin),
    };
    let mut used_radius = center_body.radius;
    bodies.push(center_body);

    // Add bodies.
    loop {
        let radius = rng.gen_range(0.6..4.0);
        let distance = radius + used_radius + rng.gen_range(1.0..32.0);
        let orbit_speed =
            Orbit::DEFAULT_ORBIT_SPEED / distance * rng.gen_range(0.5..2.0) * (rng.gen::<f32>() - 0.5).signum();

        let new_used_radius = radius.mul_add(2.0, distance);
        if new_used_radius > max_radius {
            break;
        }
        used_radius = new_used_radius;

        bodies.push(CelestialBody {
            body_type: CelestialBodyType::Planet,
            radius,
            orbit: Orbit {
                origin,
                distance,
                start_angle: rng.gen::<f32>() * TAU,
                orbit_speed,
            },
        });
    }

    System {
        bound: used_radius,
        position,
        bodies,
        infos: Vec::new(),
        colony: Vec::new(),
    }
}
