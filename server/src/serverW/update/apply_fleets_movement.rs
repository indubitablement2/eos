use common::orbit::Orbit;

use crate::serverW::*;

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity, friction and orbit.
pub fn apply_fleets_movement(s: &mut Server) {
    let bound_squared = s.server_configs.metascape_configs.bound.powi(2);
    let timef = s.time.as_timef();

    let (position, wish_position, velocity, in_system, idle_counter, acceleration, orbit) = query_ptr!(
        s.fleets,
        Fleet::position,
        Fleet::wish_position,
        Fleet::velocity,
        Fleet::in_system,
        Fleet::idle_counter,
        Fleet::acceleration,
        Fleet::orbit,
    );

    for i in 0..s.fleets.len() {
        let (position, wish_position, velocity, in_system, idle_counter, acceleration, orbit) = unsafe {
            (
                &mut *position.add(i),
                &mut *wish_position.add(i),
                &mut *velocity.add(i),
                &*in_system.add(i),
                &mut *idle_counter.add(i),
                &*acceleration.add(i),
                &mut *orbit.add(i),
            )
        };

        if let Some(target) = wish_position.target() {
            // Go toward our target.

            let wish_vel = target - *position - *velocity;
            let wish_vel_lenght = wish_vel.length();

            // Seek target.
            *velocity +=
                wish_vel.clamp_length_max(*acceleration * wish_position.movement_multiplier());

            // Stop if we are near the target.
            // TODO: Test this stop threshold.
            if wish_vel_lenght < *acceleration {
                wish_position.stop();
            }

            idle_counter.reset();
            *orbit = None;
        } else if velocity.x != 0.0 || velocity.y != 0.0 {
            // Go against our current velocity.

            let vel_change = -velocity.clamp_length_max(*acceleration);
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
                    let (system_position, system_planets) = query!(
                        s.systems,
                        s.systems.get_index(*system_id).unwrap(),
                        System::position,
                        System::planets,
                    );

                    let relative_position = *position - *system_position;
                    let distance = relative_position.length();

                    // Check if there is a planet nearby we should copy its orbit speed.
                    // Otherwise we will take a stationary orbit (0 speed).
                    let mut orbit_speed = 0.0;
                    system_planets.iter().fold(999.0f32, |closest, planet| {
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
                        *system_position,
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
