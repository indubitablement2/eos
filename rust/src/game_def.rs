use serde::{Deserialize, Serialize};
use gdnative::{api::*, prelude::*};
use std::convert::TryFrom;
use crate::constants::*;
use crate::range::*;

#[derive(Serialize, Deserialize)]
pub struct GameDef {
    pub monsters: Vec<Monster>,
    /// The individual sprite location.
    pub sprites_paths: Vec<String>,
    /// The order in which mods are loaded.
    pub mod_order: Vec<String>,
}
impl GameDef {
    pub fn new(world_path: &String) -> Self {
        let file = File::new();
        let dir = Directory::new();

        // Load game_def if it already exist.
        let game_def_path = format!("{}game_def", world_path);
        if file.open(&game_def_path, File::READ).is_ok() {
            let num_byte = file.get_64();
            let data = file.get_buffer(num_byte);
            let data_read = data.read();
            if let Ok(game_def) = bincode::deserialize::<GameDef>(&data_read) {
                return game_def;
            } else {
                godot_warn!("Found game_def, but could not deserialize it.");
            }
        }
        file.close();

        let mut game_def = GameDef {
            monsters: Vec::with_capacity(1024),
            sprites_paths: Vec::with_capacity(1024),
            mod_order: load_mod_order(world_path),
        };

        // Make a new game_def.
        game_def.mod_order.clone().iter().for_each(|mod_name| {
            let mut mod_path = String::new();
            if mod_name.as_str() == APP_NAME {
                // The base game is in res://base/.
                mod_path = BASE_GAME.to_string();
            } else {
                // A regular mmod is in user://mods/
                mod_path = format!("{}{}", MODS_PATH, mod_name);
            }

            if dir.open(&mod_path).is_ok() {
                
            } else {
                godot_error!("Could not open mod at {}.", mod_path); // TODO: Missing mod cancel loading world.
            }
        });
        
        // Save game_def.
        if file.open(&game_def_path, File::WRITE).is_ok() {
            if let Ok(data) = bincode::serialize(&game_def) {
                let num_byte = i64::try_from(data.len()).unwrap_or_default();
                file.store_64(num_byte);
                file.store_buffer(TypedArray::from_vec(data));
            } else {
                godot_error!("Could not serialize game_def to save it.");
            }
        } else {
            godot_error!("Could not open {} to save game_def.", &game_def_path);
        }
        file.close();

        game_def
    }
}

/// Load mod order or return a default one with just chaos_cascade.
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
        mod_order.push(APP_NAME.to_string());
        godot_error!("Could not open {}.", mod_order_path);
    }

    file.close();

    mod_order
}

#[derive(Serialize, Deserialize)]
pub struct Monster {
    description: String,
    default_faction: u32,
    size: f32,
    anim_idle: u32,
    anim_walk: u32,
    max_hp: i32,
    speed: i32,
    material: Vec<u32>,
    aggression: i32,
    morale: i32,
    dodge: i32,
    vision_day: i32,
    vision_night: i32,
    death_drop: Vec<(u32, )>,
    // TODO: Damage/armor.
}

pub struct DeathDrop {
    item: u32,
    quantity_range: RangeI32,
    chance: f32,
}
