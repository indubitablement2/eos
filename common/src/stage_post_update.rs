use crate::{ecs_components::*, res_parameters::ParametersRes};
use bevy_ecs::prelude::*;
// use bevy_tasks::TaskPool;

pub fn add_systems(schedule: &mut Schedule) {
    let current_stage = "post_update";
    let previous_stage = "update";

    schedule.add_stage_after(previous_stage, current_stage, SystemStage::parallel());

    schedule.add_system_to_stage(current_stage, apply_velocity.system());
}


fn apply_velocity(query: Query<(&mut Position, &mut Velocity)>, params: Res<ParametersRes>) {
    query.for_each_mut(|(mut pos, mut vel)| {
        // Apply velocity.
        pos.0 += vel.0;

        // Apply friction.
        vel.0 *= params.movement_friction;
    });
}