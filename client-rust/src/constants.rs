use gdnative::core_types::Color;

/// How many godot unit is equal to one game unit.
pub const GAME_TO_GODOT_RATIO: f32 = 128.0;

pub const SYSTEMS_FILE_PATH: &str = "res://data/systems.bin";

pub const COLOR_ALICE_BLUE: Color = Color {
    r: 0.94,
    g: 0.97,
    b: 1.0,
    a: 1.0,
};

pub const COLOR_WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
