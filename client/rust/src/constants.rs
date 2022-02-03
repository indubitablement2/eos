use gdnative::core_types::Color;

/// How many godot unit is equal to one game unit.
pub const GAME_TO_GODOT_RATIO: f32 = 256.0;

pub const FACTIONS_FILE_PATH: &str = "res://data/factions.yaml";
pub const SYSTEMS_FILE_PATH: &str = "res://data/systems.bin";

pub const COLOR_ALICE_BLUE: Color = Color {
    r: 0.94,
    g: 0.97,
    b: 1.0,
    a: 1.0,
};
