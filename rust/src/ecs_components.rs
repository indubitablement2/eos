use gdnative::core_types::Vector2;

/// A location inside the chunk grid.
pub struct Location {
    pub location_on_chunk: i64,
    pub location_on_tile: Vector2,
}

/// Always tends toward 0.
pub struct Velocity {
    pub vel: Vector2
}

pub struct Path {
    // path array
}