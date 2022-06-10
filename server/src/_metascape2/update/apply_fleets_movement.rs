use crate::_metascape2::*;

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity, friction and orbit.
pub fn apply_fleets_movement(s: &mut Metascape) {
    let bound_squared = s.systems.bound.powi(2);
    let timef = time().as_timef();

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
        let acceleration = fleet_inner.fleet_stats().acceleration;

        if let Some(target) = wish_position.target() {
            // Go toward our target.

            let wish_vel = target - *position - *velocity;
            let wish_vel_lenght = wish_vel.length();

            // Seek target.
            *velocity +=
                wish_vel.clamp_length_max(acceleration * wish_position.movement_multiplier());

            // Stop if we are near the target.
            // TODO: Test this stop threshold.
            if wish_vel_lenght < acceleration {
                wish_position.stop();
            }

            idle_counter.reset();
            *orbit = None;
        } else if velocity.x != 0.0 || velocity.y != 0.0 {
            // Go against our current velocity.

            let vel_change = -velocity.clamp_length_max(acceleration);
            *velocity += vel_change;

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

            // Apply friction.
            *velocity *= s.server_configs.metascape_configs.friction;

            // Apply velocity.
            *position += *velocity;
        }
    }
}
