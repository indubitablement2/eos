use gdnative::core_types::Vector2;

/// Entity's current chunk.
pub struct ChunkLocation {
    pub current_chunk: u32,
}

/// Entity's current tile.
pub struct TileLocation {
    pub current_tile: u32,
}

/// Position relative to current chunk center.
pub struct Position {
    pub position: Vector2,
}

/// Always tends toward 0.
pub struct Velocity {
    pub velocity: Vector2,
}

/// Activate chunks around it. Used on players.
pub struct ChunkInvoker {}
