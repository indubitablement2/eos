use crate::battlescape_components::*;
use crate::battlescape_resources::*;
use bevy_ecs::prelude::*;
use rapier2d::prelude::*;

pub fn time_system(mut time: ResMut<TimeRes>) {
    time.tick += 1;
}

// pub fn apply_velocity(query: Query<(&Velocity, &mut Position)>) {
//     query.for_each_mut(|(velocity, mut position)| {
//         position.position += velocity.velocity;
//     });
// }

pub fn physic_system(
    mut physic_pipeline: ResMut<PhysicsPipelineRes>,
    integration_parameters: Res<IntegrationParametersRes>,
    mut islands: ResMut<IslandManagerRes>,
    mut broad_phase: ResMut<BroadPhaseRes>,
    mut narrow_phase: ResMut<NarrowPhaseRes>,
    mut bodies: ResMut<BodySetRes>,
    mut colliders: ResMut<ColliderSetRes>,
    mut joints: ResMut<JointSetRes>,
    mut ccd_solver: ResMut<CCDSolverRes>,
    events: ResMut<EventCollectorRes>,
) {
    physic_pipeline.0.step(
        &vector![0.0f32, 0.0f32],
        &integration_parameters.0,
        &mut islands.0,
        &mut broad_phase.0,
        &mut narrow_phase.0,
        &mut bodies.0,
        &mut colliders.0,
        &mut joints.0,
        &mut ccd_solver.0,
        &(),
        &events.0,
    )
}

/// TODO: Draw order.
pub fn draw_system(
    render_res: Res<RenderRes>, 
    body_set: Res<BodySetRes>,
    query_physic: Query<(&Renderable, &PhysicBodyHandle)>, 
    _query: Query<(&Renderable, &Position)>
) {
    let mut render_data = gdnative::core_types::Float32Array::new();
    render_data.resize(render_res.multimesh_allocate);

    // Number of visible instances.
    let mut instances = 0i64;

    {
        let mut render_data_write = render_data.write();
        let mut i = 0;
        query_physic.for_each(|(_renderable, body_handle)| {
            // transform 8, color 0, custom 4
            let body_mat = body_set.0.get(body_handle.0).unwrap().position().to_matrix();
            
            // Write transform.
            render_data_write[i] = body_mat[(0, 0)];
            render_data_write[i+1] = body_mat[(0, 1)];
            // render_data_write[i+2] = 0.0;
            render_data_write[i+3] = body_mat[(0, 2)];
            render_data_write[i+4] = body_mat[(1, 0)];
            render_data_write[i+5] = body_mat[(1, 1)];
            // render_data_write[i+6] = 0.0;
            render_data_write[i+7] = body_mat[(1, 2)];

            // TODO: Color.

            // Write custom data.
            render_data_write[i+8] = f32::from_bits(0);
            // render_data_write[i+9] = 0.0;
            // render_data_write[i+11] = 0.0;
            // render_data_write[i+12] = 0.0;

            i += 13;
            instances += 1;
        });
    }

    let visual_server = unsafe { gdnative::api::VisualServer::godot_singleton() };
    visual_server.multimesh_set_as_bulk_array(render_res.multimesh_rid, render_data);
    visual_server.multimesh_set_visible_instances(render_res.multimesh_rid, instances);
}