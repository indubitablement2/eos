// use bevy_ecs::prelude::*;
use gdnative::core_types::TypedArray;

/// Modify the game.
pub struct GameParameterRes {
    pub drag: f32, // Velocity is multiplied by this each tick.
}

pub struct TimeRes {
    pub tick: u32,
}

// ! Physic

// ! Render

/// All that is needed to render sprites.
pub struct RenderRes {
    pub render_data: Option<TypedArray<f32>>,
    pub visible_instance: i64,
}
