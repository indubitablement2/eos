use serde::{Deserialize, Serialize};
use gdnative::{api::*, prelude::*};

#[derive(Serialize, Deserialize)]
pub struct GameDef {
    pub pawns: Vec<Pawn>,
}
impl GameDef {
    pub fn new(world_path: &String) -> (Self, Vec<String>) {
        let mod_order = load_mod_order(world_path);
        let mut sprites_paths = Vec::with_capacity(1024);

        // Load base game.

        (
            GameDef {
                pawns: Vec::new(),
            },
            sprites_paths
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct Pawn {
    sprite: u32,
    max_hp: i32,
}

/// Load sprite atlas order.
fn load_mod_order(world_path: &String) -> Vec<String> {
    let mut mod_order = Vec::new();

    let file = File::new();

    let mod_order_path = format!("{}mod_order", &world_path);

    if file.open(&mod_order_path, File::READ).is_ok() {
        let mut line = file.get_line().to_string();
        while !line.is_empty() {
            mod_order.push(line);
            line = file.get_line().to_string();
        }
    } else {
        godot_error!("Could not open {}.", mod_order_path);
    }

    file.close();

    mod_order
}