use crate::ecs_resources::*;
use crate::constants::*;
use bevy_ecs::prelude::*;

pub fn time_system(mut time: ResMut<TimeRes>, param: Res<GameParameterRes>) {
    time.time += time.delta;
    if time.time >= param.day_lenght {
        time.time -= param.day_lenght;
        time.days += 1;
    }
}

pub fn prepare_render(mut render_res: ResMut<RenderRes>) {
    if render_res.render_data.len() == NUM_RENDER * DATA_PER_INSTANCE {
        render_res.render_data.resize(NUM_RENDER * 12);
    }
}

// pub fn apply_velocity(query: Query<(&Velocity, &mut Position)>) {
//     query.for_each_mut(|(velocity, mut position)| {
//         position.position += velocity.velocity;
//     });
// }

// /// TODO: Draw order.
// pub fn draw_system(
//     mut render_res: ResMut<RenderRes>,
//     body_set: Res<BodySetRes>,
//     query_physic: Query<(&Renderable, &PhysicBodyHandle)>,
//     _query: Query<(&Renderable, &Position)>,
// ) {
//     if let Some(render_data) = &mut render_res.render_data {
//         let mut instances = 0;
//         {
//             let mut render_data_write = render_data.write();
//             let mut i = 0;
//             query_physic.for_each(|(_renderable, body_handle)| {
//                 // transform 8, color 0, custom 4
//                 let body_mat = body_set.0.get(body_handle.0).unwrap().position().to_matrix();

//                 // Write transform.
//                 render_data_write[i] = body_mat[(0, 0)];
//                 render_data_write[i + 1] = body_mat[(0, 1)];
//                 // render_data_write[i+2] = 0.0;
//                 render_data_write[i + 3] = body_mat[(0, 2)];
//                 render_data_write[i + 4] = body_mat[(1, 0)];
//                 render_data_write[i + 5] = body_mat[(1, 1)];
//                 // render_data_write[i+6] = 0.0;
//                 render_data_write[i + 7] = body_mat[(1, 2)];

//                 // TODO: Color.

//                 // Write custom data.
//                 render_data_write[i + 8] = f32::from_bits(0);
//                 // render_data_write[i+9] = 0.0;
//                 // render_data_write[i+11] = 0.0;
//                 // render_data_write[i+12] = 0.0;

//                 i += 13;
//                 instances += 1;
//             });
//         }

//         render_res.visible_instance = instances;
//     }

// ! DO NOT NEED TO fill the rest of the render data array with 0 as extra instance are invisible.

// }


/// Send the render data to Godot for rendering.
pub fn render_finalize(render_res: Res<RenderRes>) {
    if render_res.render_data.len() == NUM_RENDER * DATA_PER_INSTANCE {
        let visual_server = unsafe { gdnative::api::VisualServer::godot_singleton() };

        visual_server.multimesh_set_as_bulk_array(render_res.multimesh_rid, render_res.render_data.clone());
        visual_server.multimesh_set_visible_instances(render_res.multimesh_rid, render_res.visible_instance);
        visual_server.canvas_item_add_multimesh(
            render_res.canvas_rid,
            render_res.multimesh_rid,
            render_res.texture_rid,
            render_res.normal_texture_rid,
        );
    }
}