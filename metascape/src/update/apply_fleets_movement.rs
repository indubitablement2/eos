use crate::*;
use common::timef::TimeF;

/// Update velocity based on wish position and acceleration.
///
/// Apply velocity and orbit.
pub fn apply_fleets_movement<C>(s: &mut Metascape<C>)
where
    C: ConnectionsManager,
{
    let metascape_configs = &s.configs.metascape_configs;
    let systems = &s.systems;

    let fleets_position = s.fleets.container.position.as_mut_slice();
    let fleets_wish_position = s.fleets.container.wish_position.as_mut_slice();
    let fleets_velocity = s.fleets.container.velocity.as_mut_slice();
    let fleets_in_system = s.fleets.container.in_system.as_slice();
    let fleets_idle_counter = s.fleets.container.idle_counter.as_mut_slice();
    let fleets_fleet_inner = s.fleets.container.fleet_inner.as_slice();
    let fleets_orbit = s.fleets.container.orbit.as_mut_slice();

    let bound_squared = (systems.bound + metascape_configs.systems_bound_padding).powi(2);
    let orbit_time = TimeF::tick_to_orbit_time(tick());
    let break_acceleration_multiplier = metascape_configs.break_acceleration_multiplier;
    let absolute_max_speed = metascape_configs.absolute_max_speed;

    for (
        (((((position, wish_position), velocity), in_system), idle_counter), fleet_inner),
        orbit,
    ) in fleets_position
        .into_iter()
        .zip(fleets_wish_position)
        .zip(fleets_velocity)
        .zip(fleets_in_system)
        .zip(fleets_idle_counter)
        .zip(fleets_fleet_inner)
        .zip(fleets_orbit)
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
                    let system = systems.systems.get(system_id).unwrap();

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

                    *orbit = Some((
                        Orbit::from_relative_position(
                            relative_position,
                            orbit_time,
                            system.position,
                            distance,
                            orbit_speed,
                        ),
                        tick(),
                    ));
                } else {
                    // Take a stationary orbit.
                    *orbit = Some((Orbit::stationary(*position), tick()));
                }
            }
        }

        // Update position.
        if let Some((orbit, _)) = orbit {
            // Apply orbit.
            *position = orbit.to_position(orbit_time);
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
