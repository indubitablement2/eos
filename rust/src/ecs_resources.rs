// use bevy_ecs::prelude::*;
use gdnative::core_types::*;
use gdnative::api::*;

use crate::constants::NUM_RENDER;

/// Modify the game.
pub struct GameParameterRes {
    /// Velocity is multiplied by this each tick.
    pub drag: f32,
    /// How many seconds a day last.
    pub day_lenght: f64,
}
impl Default for GameParameterRes {
    fn default() -> Self {
        Self {
            drag: 0.75,
            day_lenght: 86400.0,
        }
    }
}

pub struct TimeRes {
    /// Number of days elapsed.
    pub days: u32,
    /// Time in seconds for this day.
    pub time: f64,
    /// Time elapsed since last update.
    pub delta: f64,
}
impl Default for TimeRes {
    fn default() -> Self {
        Self {
            days: 0,
            time: 0.0,
            delta: 0.0,
        }
    }
}

// ! Physic

// ! Render

/// All that is needed to render sprites.
pub struct RenderRes {
    pub canvas_rid: Rid,
    pub mesh_rid: Rid,
    pub multimesh_rid: Rid,
    // pub texture: Ref<TextureArray>,
    pub texture_rid: Rid,
    /// Unused.
    pub normal_texture_rid: Rid,
    /// This is the bulk array that is needed for the multimesh.
    pub render_data: TypedArray<f32>,
    /// This is used to cull extra uneeded sprite from render_data.
    pub visible_instance: i64,
}
impl RenderRes {
    pub fn new(canvas_rid: Rid, texture_rid: Rid) -> Self {
        let visual_server = unsafe { gdnative::api::VisualServer::godot_singleton() };

        // Create mesh.
        let mesh_rid = visual_server.mesh_create();
        crate::render_util::init_basic_mesh(visual_server, mesh_rid);

        // Create multimesh.
        let multimesh_allocate = NUM_RENDER;
        let multimesh_rid = visual_server.multimesh_create();
        visual_server.multimesh_set_mesh(multimesh_rid, mesh_rid);
        visual_server.multimesh_allocate(
            multimesh_rid,
            multimesh_allocate.into(),
            VisualServer::MULTIMESH_TRANSFORM_2D,
            VisualServer::MULTIMESH_COLOR_NONE,
            VisualServer::MULTIMESH_CUSTOM_DATA_FLOAT,
        );

        Self {
            canvas_rid,
            mesh_rid,
            multimesh_rid,
            texture_rid,
            normal_texture_rid: Rid::new(),
            render_data: TypedArray::new(),
            visible_instance: 0,
        }
    }
}