pub const APP_NAME: &str = "chaos_cascade";
pub const BASE_MOD_PATH: &str = "res://base/";
pub const ERR_SPRITE_PATH: &str = "res://error.png";

/// Maximum number of renderable sprites.
pub const NUM_RENDER: i32 = 30000;
/// Matrix2 = 8 + custom_8bit = 1
pub const DATA_PER_INSTANCE: i32 = 9;
pub const BULK_ARRAY_SIZE: i32 = NUM_RENDER * DATA_PER_INSTANCE;
/// Sprite atlas size is a power of 2 at or above this size.
pub const MIN_SPRITE_ATLAS_SIZE: i64 = 2048;
pub const MAX_SPRITE_ATLAS: usize = 15;

/// Rotation prefix used to auto detect rotation sprite.
pub const ROT_NAME: [&str; 8] = ["_b", "_bl", "_l", "_tl", "_t", "_tr", "_r", "_br"];
/// Sprite retation replacement order.
pub const ROT_REPLACEMENT: [[usize; 7]; 8] = [
    [1, 7, 2, 6, 3, 5, 4], // 0
    [2, 0, 3, 7, 4, 6, 5], // 1
    [1, 3, 0, 4, 7, 6, 5], // 2
    [2, 1, 4, 0, 5, 7, 6], // 3
    [3, 5, 2, 6, 1, 7, 0], // 4
    [6, 7, 4, 0, 3, 1, 2], // 5
    [7, 5, 0, 4, 1, 2, 3], // 6
    [6, 0, 5, 1, 4, 2, 3], // 7
];

/// TODO: Delete
pub const MAX_FLOATING_ORIGIN_DISTANCE_TO_PLAYER: f32 = 10000.0;

pub const MODS_PATH: &str = "user://mods/";
pub const CACHE: &str = "user://caches/";
pub const WORLDS_PATH: &str = "user://worlds/";
pub const CHARACTERS_PATH: &str = "user://characters/";
