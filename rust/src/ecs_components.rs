use glam::Vec2;

/// Entity's current tile.
pub struct TileLocation {
    pub current_tile: u32,
}

/// Position relative to player's current tile.
pub struct Position {
    pub position: Vec2,
}

/// Always tends toward 0.
pub struct Velocity {
    pub velocity: Vec2,
}

pub struct Sprite {
    
}

// /// On which layers this entity reside.
// pub struct PhysicLayer {
//     pub layer_mask: u32,
// }
// /// On which layers this entity scan for collision.
// pub struct PhysicCollideLayer {
//     pub collide_layer_mask: u32,
// }
