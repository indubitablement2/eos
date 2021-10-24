use bevy_ecs::prelude::Entity;
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
