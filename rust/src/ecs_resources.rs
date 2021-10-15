use crate::constants::*;
use bevy_ecs::prelude::Entity;
use gdnative::api::*;
use gdnative::core_types::*;
use glam::DVec2;
use glam::IVec2;
use std::convert::TryFrom;

/// Modify the game.
pub struct GameParameterRes {
    /// Velocity is multiplied by this each tick.
    pub drag: f32,
    /// How many real seconds a day last.
    pub day_lenght: f32,
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
    /// Time in real seconds for this day.
    pub time: f32,
    /// Time elapsed since last update.
    pub delta: f32,
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

pub struct FloatingOriginRes {
    /// Floating origin position relative to true origine (0.0, 0.0).
    pub floating_origin_position: DVec2,
    /// Floating origin tile location.
    pub floating_origin_tile: IVec2,
}

/// Tiles that are loaded. This is like a rect2 on all possible tiles.
pub struct LoadedChunkRes {
    /// First tile in this chunk.
    pub position_start: IVec2,
    /// Dimention of the chunk.
    pub extend: IVec2,
    /// Helper derived from position and extend.
    width: i32,
    /// Loaded tiles are stored in a contiguous array.
    pub tiles: Vec<()>, // TODO
}
impl LoadedChunkRes {
    /// Return the tile index inside tiles array. Value is clamped if outside this chunk.
    #[inline]
    pub fn get_tile_index(&self, tile: IVec2) -> usize {
        let relative_position = tile - self.position_start;
        usize::try_from(relative_position.x + relative_position.y * self.width)
            .unwrap_or_default()
            .min(self.tiles.len())
    }
}

pub struct PlayerRes {
    pub entity: Entity,
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
        crate::utils::init_basic_mesh(visual_server, mesh_rid);

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

        let mut render_data = TypedArray::new();
        render_data.resize(NUM_RENDER * DATA_PER_INSTANCE);

        Self {
            canvas_rid,
            mesh_rid,
            multimesh_rid,
            texture_rid,
            normal_texture_rid: Rid::new(),
            render_data,
            visible_instance: 0,
        }
    }
}
