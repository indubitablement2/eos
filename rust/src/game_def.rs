use crate::constants::*;
use ahash::AHashMap;
use gdnative::{api::Directory, prelude::*};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize)]
pub struct GameDef {
    pub monsters: Vec<Monster>,
    /// The individual sprite location.
    pub sprites_paths: Vec<String>,
    /// The order in which mods are loaded.
    pub mod_order: Vec<String>,
}
impl GameDef {
    /// Attempt to load game_def or create a new one.
    pub fn load(world_path: &str) -> Self {
        let file = gdnative::api::File::new();
        
        // Load game_def if it already exist.
        let game_def_path = format!("{}game_def", world_path);
        if file.open(&game_def_path, gdnative::api::File::READ).is_ok() {
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

        GameDef::new(world_path)
    }

    /// Make a new game_def and save it.
    pub fn new(world_path: &str) -> Self {
        let file = gdnative::api::File::new();

        let mut game_def = GameDef {
            monsters: Vec::with_capacity(1024),
            sprites_paths: Vec::with_capacity(1024),
            mod_order: load_mod_order(world_path),
        };

        let all_files = RecusiveFileSearch::recursive_find_all_file(&game_def.mod_order);



        // Save game_def.
        let game_def_path = format!("{}game_def", world_path);
        if file.open(&game_def_path, gdnative::api::File::WRITE).is_ok() {
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

struct RecusiveFileSearch {
    /// All the yaml file that were found ordered by first to last mod to load.
    pub yaml_path: Vec<GodotString>,
    /// All the png file that were found keyed by their name.
    /// eg: res://base/asset/monster/monster_001_
    pub png_path: AHashMap<String, GodotString>,
}

impl RecusiveFileSearch {
    /// Recursively find all png and yaml file from the provided mod_order.
    pub fn recursive_find_all_file(mod_order: &Vec<String>) -> Self {
        let dir = gdnative::api::Directory::new();

        let mut foler_path_to_parse = Vec::with_capacity(256);
        let mut yaml_path = Vec::with_capacity(1024);
        let mut png_path = AHashMap::with_capacity(1024);

        // Start by adding all mod folders.
        mod_order.iter().rev().for_each(|mod_name| {
            foler_path_to_parse.push(GodotString::from(get_mod_path(mod_name)));
        });

        while let Some(path) = foler_path_to_parse.pop() {
            if dir.open(path.clone()).is_ok() {
                if dir.list_dir_begin(true, true).is_ok() {
                    let mut file_name = dir.get_next();
                    while !file_name.is_empty() {
                        if dir.current_is_dir() {
                            godot_print!("found dir: {}", file_name.clone());
                            foler_path_to_parse.push(path.clone() + file_name + GodotString::from_str("/"));
                        } else if file_name.ends_with(&GodotString::from_str(".yaml")) {
                            godot_print!("found yaml: {}", file_name.clone());
                            yaml_path.push(path.clone() + file_name);
                        } else if file_name.ends_with(&GodotString::from_str(".png")) {
                            godot_print!("found png: {}", file_name.clone());
                            png_path.push(path.clone() + file_name);
                        }
                        file_name = dir.get_next();
                    }
                    dir.list_dir_end();
                } else {
                    godot_error!("Could not list dir in: {}.", path);
                }
            } else {
                // TODO: Missing mod cancel loading world.
                // This should not happen as load_mod_order() already check and remove missing mods.
                godot_error!("Could not open folder: {}.", path); 
            }
        }

        Self {
            yaml_path,
            png_path,
        }
    }
}

/// Load mod order or return a default one with just chaos_cascade.
fn load_mod_order(world_path: &str) -> Vec<String> {
    let mut mod_order = Vec::new();

    let file = gdnative::api::File::new();

    let mod_order_path = format!("{}mod_order", &world_path);

    // Try to open saved mod_order or return a default one.
    if file.open(&mod_order_path, gdnative::api::File::READ).is_ok() {
        let mut line = file.get_line().to_string();
        while !line.is_empty() {
            mod_order.push(line);
            line = file.get_line().to_string();
        }
    } else {
        godot_error!("Could not open {}.", mod_order_path);
    }
    file.close();

    // Check that each mod exist.
    let dir = Directory::new();
    mod_order.drain_filter(|mod_name| {
        let mod_path = get_mod_path(mod_name);
        if dir.dir_exists(&mod_path) {
            return false;
        }
        godot_warn!("Could not find {} at {}.", mod_name, &mod_path);
        true
    });

    // Make sure we a have at least one mod.
    if mod_order.is_empty() {
        mod_order.push(APP_NAME.to_string());
    }

    godot_print!("Loaded mod_order: {:?}.", &mod_order);

    mod_order
}

/// Return a path to the mod folder given a mod name. Does not guaranty the mod folder exist.
fn get_mod_path(mod_name: &str) -> String {
    if mod_name == APP_NAME {
        // The base game is in res://base/.
        BASE_GAME_PATH.to_string()
    } else {
        // A regular mmod is in user://mods/
        format!("{}{}", MODS_PATH, mod_name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Monster {
    name: String,
    description: String,
    default_faction: usize,

    size: f32,

    anim_idle: usize,
    anim_run: usize,
    material: usize,

    max_hp: i32,
    // TODO: Damage/armor.
    speed: f32,

    aggression: i32,
    morale: i32,

    vision_day: i32,
    vision_night: i32,

    death_drop: Vec<Drop>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Drop {
    item: usize,
    quantity: (u32, u32),
    chance: (u32, u32),
}
