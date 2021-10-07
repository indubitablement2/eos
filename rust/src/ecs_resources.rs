// use bevy_ecs::prelude::*;
use gdnative::core_types::TypedArray;

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
    /// This is the bulk array that is needed for the multimesh.
    pub render_data: TypedArray<f32>,
    /// This is used to cull extra uneeded sprite from render_data.
    pub visible_instance: i64,
}
impl Default for RenderRes {
    fn default() -> Self {
        Self {
            render_data: Default::default(),
            visible_instance: Default::default(),
        }
    }
}
