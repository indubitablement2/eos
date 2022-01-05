use crate::{orbit_to_world_position, parameters::MetascapeParameters, position::*};
use glam::Vec2;

/// If position changed, return the new world position.
pub fn compute_fleet_movement(
    position: Position,
    velocity: &mut Vec2,
    wish_position: Option<Vec2>,
    acceleration: f32,
    time: f32,
    metascape_parameters: &MetascapeParameters,
) -> Option<Vec2> {
    // Apply friction.
    *velocity *= metascape_parameters.movement_friction;

    match position {
        Position::WorldPosition { world_position } => {
            if let Some(target) = wish_position {
                // Add velocity toward target at full speed.
                *velocity += (target - world_position).clamp_length_max(acceleration);

                // Apply velocity.
                Some(world_position + *velocity)
            } else {
                if velocity.x != 0.0 || velocity.y != 0.0 {
                    // Go against current velocity.
                    let vel_change = -velocity.clamp_length_max(acceleration);
                    *velocity += vel_change;

                    // Set velocity to zero if we have nearly no velocity.
                    if velocity.x.abs() < 0.001 {
                        velocity.x = 0.0;
                    }
                    if velocity.y.abs() < 0.001 {
                        velocity.y = 0.0;
                    }

                    // Apply velocity.
                    Some(world_position + *velocity)
                } else {
                    None
                }
            }
        }
        Position::Orbit {
            origin,
            orbit_radius,
            orbit_start_angle,
            orbit_time,
        } => {
            if wish_position.is_some() || velocity.x != 0.0 || velocity.y != 0.0 {
                Some(orbit_to_world_position(
                    origin,
                    orbit_radius,
                    orbit_start_angle,
                    orbit_time,
                    time,
                ))
            } else {
                None
            }
        }
    }
}
