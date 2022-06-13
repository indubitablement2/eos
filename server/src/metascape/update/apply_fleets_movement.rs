use crate::metascape::*;

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity and orbit.
pub fn apply_fleets_movement(s: &mut Metascape) {
    let bound_squared =
        (s.systems.bound + s.server_configs.metascape_configs.systems_bound_padding).powi(2);
    let timef = time().as_timef();
    let break_acceleration_multiplier = s
        .server_configs
        .metascape_configs
        .break_acceleration_multiplier;
    let absolute_max_speed = s.server_configs.metascape_configs.absolute_max_speed;

    let position = s.fleets.container.position.iter_mut();
    let wish_position = s.fleets.container.wish_position.iter_mut();
    let velocity = s.fleets.container.velocity.iter_mut();
    let in_system = s.fleets.container.in_system.iter();
    let idle_counter = s.fleets.container.idle_counter.iter_mut();
    let fleet_inner = s.fleets.container.fleet_inner.iter();
    let orbit = s.fleets.container.orbit.iter_mut();

    for (
        (((((position, wish_position), velocity), in_system), idle_counter), fleet_inner),
        orbit,
    ) in position
        .zip(wish_position)
        .zip(velocity)
        .zip(in_system)
        .zip(idle_counter)
        .zip(fleet_inner)
        .zip(orbit)
    {
        let max_speed = fleet_inner.fleet_stats().max_speed;
        let acceleration = fleet_inner.fleet_stats().acceleration;
        let radius = fleet_inner.fleet_stats().radius;

        if let Some(relative_target) = wish_position.target().and_then(|target| {
            let relative_target = target - *position;

            if relative_target.length_squared() <= radius.powi(2) {
                wish_position.stop();
                None
            } else {
                Some(relative_target)
            }
        }) {
            // Go toward our target.

            let velocity_len = velocity.length();
            let time_to_break = velocity_len / (acceleration * break_acceleration_multiplier * 1.1);

            let wish_vel = relative_target - *velocity * time_to_break;
            *velocity +=
                wish_vel.clamp_length_max(acceleration * wish_position.movement_multiplier());

            *velocity =
                velocity.clamp_length_max(velocity_len.max(max_speed).min(absolute_max_speed));

            idle_counter.reset();
            *orbit = None;
        } else if velocity.x != 0.0 || velocity.y != 0.0 {
            // Go against our current velocity.

            *velocity -= velocity.clamp_length_max(acceleration * break_acceleration_multiplier);

            // Set velocity to zero if we have nearly no velocity.
            if velocity.x.abs() < 0.001 {
                velocity.x = 0.0;
            }
            if velocity.y.abs() < 0.001 {
                velocity.y = 0.0;
            }

            idle_counter.reset();
            *orbit = None;
        } else {
            // We are idle.

            idle_counter.increment();

            if idle_counter.is_idle() && orbit.is_none() {
                // Take an orbit as we are idle.
                if let Some(system_id) = in_system {
                    let system = s.systems.systems.get(system_id).unwrap();

                    let relative_position = *position - system.position;
                    let distance = relative_position.length();

                    // Check if there is a planet nearby we should copy its orbit speed.
                    // Otherwise we will take a stationary orbit (0 speed).
                    let mut orbit_speed = 0.0;
                    system.planets.iter().fold(999.0f32, |closest, planet| {
                        let dif = (planet.relative_orbit.distance - distance).abs();
                        if dif < closest {
                            orbit_speed = planet.relative_orbit.orbit_speed;
                            dif
                        } else {
                            closest
                        }
                    });

                    *orbit = Some(Orbit::from_relative_position(
                        relative_position,
                        timef,
                        system.position,
                        distance,
                        orbit_speed,
                    ));
                } else {
                    // Take a stationary orbit.
                    *orbit = Some(Orbit::stationary(*position));
                }
            }
        }

        // Update position.
        if let Some(orbit) = orbit {
            // Apply orbit.
            *position = orbit.to_position(timef);
        } else {
            // Fleets are pushed away from the world's bound.
            if position.length_squared() > bound_squared {
                *velocity -= position.normalize() * 8.0;
            }

            // Apply velocity.
            *position += *velocity;
        }
    }
}
