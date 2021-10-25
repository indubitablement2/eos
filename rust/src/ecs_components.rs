use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};

// #[component(storage = "SparseSet")]
// Faster add/remove, slower iteration than default.

// ! Component that directly come from mods.

/// A reference to the bundle this entity came from as well as the name(referenceId) of the bundle.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BundleReferenceId(pub usize);

// ! Components that can't directly come from mods.

/// The display name of the entity.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Name(pub String);

/// The faction of the entity.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Faction(pub usize);

/// A sprite and a color.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Sprite {
    pub sprite_id: usize,
    pub color: [f32; 4],
}

// TODO: Clean these.

/// Entity's current tile.
pub struct TileLocationComp {
    pub current_tile: IVec2,
}

/// Position relative to floating origin.
pub struct Position {
    pub position: Vec2,
}

/// Always tends toward 0.
pub struct VelocityComp {
    pub velocity: Vec2,
}

// /// On which layers this entity reside.
// pub struct PhysicLayer {
//     pub layer_mask: u32,
// }
// /// On which layers this entity scan for collision.
// pub struct PhysicCollideLayer {
//     pub collide_layer_mask: u32,
// }
