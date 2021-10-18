use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};

// ! Component that can come from mods.

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Name(String);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct DefaultFaction(u32);

// ! Components that can't be serialized.

/// Entity's current tile.
pub struct TileLocationComp {
    pub current_tile: IVec2,
}

/// Position relative to floating origin.
pub struct PositionComp {
    pub position: Vec2,
}

/// Always tends toward 0.
pub struct VelocityComp {
    pub velocity: Vec2,
}

pub struct SpriteComp {
    pub sprite_id: u32,
    pub color: (f32, f32, f32),
}

// /// On which layers this entity reside.
// pub struct PhysicLayer {
//     pub layer_mask: u32,
// }
// /// On which layers this entity scan for collision.
// pub struct PhysicCollideLayer {
//     pub collide_layer_mask: u32,
// }
