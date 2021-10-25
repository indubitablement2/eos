use crate::constants::*;
use crate::ecs_components::*;
use crate::ecs_render_pipeline::RenderRes;
use crate::ecs_resources::*;
use bevy_ecs::prelude::*;
use glam::Vec2;

pub fn time_system(mut time_res: ResMut<TimeRes>, param_res: Res<GameParameterRes>) {
    time_res.time += time_res.delta;
    if time_res.time >= param_res.day_lenght {
        time_res.time -= param_res.day_lenght;
        time_res.days += 1;
    }
}

/// Move the floating origin to the player if it past a threshold.
/// TODO: Move floating origin to the center of players cluster for multiplayer.
pub fn move_floating_origin(
    player_res: Res<PlayerRes>,
    mut floating_origin_res: ResMut<FloatingOriginRes>,
    mut query: Query<&mut Position>,
) {
    // Find how far the player is to the floating origin (this is just his position in single player).
    let mut difference = Vec2::ZERO;
    if let Ok(player_position) = query.get_mut(player_res.entity) {
        difference = player_position.position;
    }

    if difference.length() > MAX_FLOATING_ORIGIN_DISTANCE_TO_PLAYER {
        // TODO: Move origin position and tile.
        floating_origin_res.floating_origin_position += difference.as_dvec2();

        // Move every entity with position.
        query.for_each_mut(|mut pos| {
            pos.position -= difference;
        });

        // godot_print!(
        //     "Moved floating origin to {:?}. Difference: {:?}.",
        //     &floating_origin_res.floating_origin_position,
        //     difference
        // );
    }
}

/// TODO: Draw order.
pub fn draw_sprite_system(mut render_res: ResMut<RenderRes>, query: Query<(&Sprite, &Position)>) {
    let mut instances = 0;
    
    {
        let render_data_write = render_res.render_data.write();
        query.for_each(|(sprite, pos)| {
            render_res.render_data
        });
    }


    // let mut i = 0; 
    // query_physic.for_each(|(_renderable, body_handle)| {
    //     // transform 8, color 0, custom 4
    //     let body_mat = body_set.0.get(body_handle.0).unwrap().position().to_matrix();

    //     // Write transform.
    //     render_data_write[i] = body_mat[(0, 0)];
    //     render_data_write[i + 1] = body_mat[(0, 1)];
    //     // render_data_write[i+2] = 0.0;
    //     render_data_write[i + 3] = body_mat[(0, 2)];
    //     render_data_write[i + 4] = body_mat[(1, 0)];
    //     render_data_write[i + 5] = body_mat[(1, 1)];
    //     // render_data_write[i+6] = 0.0;
    //     render_data_write[i + 7] = body_mat[(1, 2)];

    //     // TODO: Color.

    //     // Write custom data.
    //     render_data_write[i + 8] = f32::from_bits(0);
    //     // render_data_write[i+9] = 0.0;
    //     // render_data_write[i+11] = 0.0;
    //     // render_data_write[i+12] = 0.0;

    //     i += 13;
    //     instances += 1;
    // });

    render_res.visible_instance = instances;
}

/// TODO: Fetch sprite entity and make a bulk data array.
pub fn render_prepare_sprites() {
    todo!()
}

// /// Send the render data to Godot for rendering.
// pub fn render_finalize(render_res: Res<RenderRes>) {
// let visual_server = unsafe { gdnative::api::VisualServer::godot_singleton() };

// visual_server.multimesh_set_as_bulk_array(render_res.multimesh_rid, render_res.render_data.clone());
// visual_server.multimesh_set_visible_instances(render_res.multimesh_rid, render_res.visible_instance);
// visual_server.canvas_item_add_multimesh(
//     render_res.canvas_rid,
//     render_res.multimesh_rid,
//     render_res.texture_rid,
//     render_res.normal_texture_rid,
// );
// }
