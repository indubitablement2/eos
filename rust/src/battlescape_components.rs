use gdnative::core_types::Vector2;
use rapier2d::prelude::*;

/// Entity's current tile.
pub struct TileLocation {
    pub current_tile: u32,
}

/// Position relative to map top left corner.
pub struct Position {
    pub position: Vector2,
}

/// Always tends toward 0.
pub struct Velocity {
    pub velocity: Vector2,
}

pub struct Renderable {}

pub struct PhysicBodyHandle(pub RigidBodyHandle);
pub struct PhysicCollisionHandle(pub ColliderHandle);
// /// On which layers this entity reside.
// pub struct PhysicLayer {
//     pub layer_mask: u32,
// }
// /// On which layers this entity scan for collision.
// pub struct PhysicCollideLayer {
//     pub collide_layer_mask: u32,
// }
