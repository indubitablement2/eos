use bevy_ecs::prelude::*;
use crate::ecs_components::*;


pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "update";
    let previous_stage = "pre_update";

    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());

    schedule.add_system_to_stage(current_stage, movement.system());
}

/// Add velocity based on wish position.
/// TODO: Fleets engaged in the same Battlescape should aggregate.
fn movement(mut query: Query<(&Position, &WishPosition, &mut Velocity)>) {
    query.for_each_mut(|(pos, wish_pos, mut vel)| {
        // TODO: Stop threshold.
        if pos.0.distance_squared(wish_pos.0) < 10.0 {
            // Try to stop.
            let new_vel = -vel.0.clamp_length_max(1.0);
            vel.0 += new_vel;
        } else {
            // Add velocity toward fleet's wish position at full speed.
            vel.0 += (wish_pos.0 - pos.0).clamp_length_max(1.0);
        }
    });
}