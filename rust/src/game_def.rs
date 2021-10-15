use crate::constants::*;
use ahash::AHashMap;
use gdnative::prelude::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, hash::{Hash, Hasher}, str::pattern::Pattern};

#[derive(Serialize, Deserialize)]
pub struct GameDef {
    pub monsters: Vec<Monster>,
    /// The individual sprite location.
    pub sprites_paths: Vec<String>,
    /// The order in which mods are loaded.
    pub mod_order: Vec<ModInfo>,
}
impl GameDef {
    /// Load a cached game_def or create a new one.
    pub fn load(world_path: &str, try_load_from_cache: bool, save_if_new: bool) -> Self {
        let file = gdnative::api::File::new();

        // Get mod order.
        let mut mod_order = vec![ModInfo::default()];
        let mod_order_path = format!("{}mod_order", &world_path);
        if file.open(&mod_order_path, gdnative::api::File::READ).is_ok() {
            let data = file.get_as_text().to_string();
            if let Ok(loaded_mod_order) = serde_yaml::from_str::<Vec<ModInfo>>(&data) {
                mod_order = loaded_mod_order;
                godot_print!("Loaded mod order:\n{:?}", &mod_order);
            } else {
                godot_warn!("Could not deserialize loaded mod_order. Using default instead.");
            }
        } else {
            godot_warn!("Could not open {}. Using default instead.", mod_order_path);
        }
        file.close();

        // Hash mod_order to string. This will be the name of the cached game_def.
        let mut hasher = ahash::AHasher::new_with_keys(1667, 420);
        mod_order.hash(&mut hasher);
        let hashed_mod_order = hasher.finish();

        let game_def_path = format!("{}{}", MOD_CACHE, format!("{:x}", hashed_mod_order));

        // Load cached game_def if it already exist.
        if try_load_from_cache {
            if file.open(&game_def_path, gdnative::api::File::READ).is_ok() {
                let num_byte = file.get_64();
                let data = file.get_buffer(num_byte);
                let data_read = data.read();
                if let Ok(game_def) = bincode::deserialize::<GameDef>(&data_read) {
                    if game_def.mod_order == mod_order {
                        godot_print!("Found cached game def.");
                        file.close();
                        return game_def;
                    } else {
                        godot_error!(
                            "Found matching cached game_def, but mor_order don't match.\n{:?}\n{:?}",
                            &game_def.mod_order,
                            &mod_order
                        );
                    }
                } else {
                    godot_error!("Found game_def, but could not deserialize it.");
                }
            }
            file.close();

            // We did not find a cached game_def.
            godot_print!("Did not find cached game_def. Making and caching a new one.");
        }

        let game_def = GameDef::new(mod_order);
        

        // Save game_def for faster load time.
        if save_if_new {
            if file.open(&game_def_path, gdnative::api::File::WRITE).is_ok() {
                if let Ok(data) = bincode::serialize(&game_def) {
                    let num_byte = i64::try_from(data.len()).unwrap_or_default();
                    file.store_64(num_byte);
                    file.store_buffer(TypedArray::from_vec(data));
                } else {
                    godot_error!("Could not serialize game_def to cache it.");
                }
            } else {
                godot_error!("Could not open {} to cahce game_def.", &game_def_path);
            }
            file.close();
        }

        game_def
    }

    fn new(mod_order: Vec<ModInfo>) -> Self {
        let dir = gdnative::api::Directory::new();

        let mut folder_path_to_parse = Vec::with_capacity(256);
        
        // All the yaml file that were found ordered by first to last mod to load.
        let mut yaml_path = Vec::with_capacity(1024);

        // All the png file that were found keyed by their name.
        // eg: res://base/asset/monster/monster_walk_00_#.png
        // Last number is important as it the number of oriantation.
        // In the above example, the sprite name would be asset/monster/monster_walk_00
        // TODO
        let mut png_path = AHashMap::with_capacity(1024);

        // Start by adding all mod folders.
        mod_order.iter().rev().for_each(|mod_info| {
            folder_path_to_parse.push(mod_info.get_path());
        });

        while let Some(path) = folder_path_to_parse.pop() {
            if dir.open(path).is_ok() {
                if dir.list_dir_begin(true, true).is_ok() {
                    let mut file_name = dir.get_next().to_string();
                    while !file_name.is_empty() {
                        if dir.current_is_dir() {
                            godot_print!("found dir: {}", &file_name);
                            folder_path_to_parse.push(format!("{}{}/", &path, &file_name));
                        } else if file_name.ends_with(".yaml") {
                            godot_print!("found yaml: {}", &file_name);
                            yaml_path.push(format!("{}{}", &path, &file_name));
                        } else if let Some(file_parsed_name) = file_name.strip_suffix(".png") {
                            godot_print!("found png: {}", &file_name);
                            // Here we need to parse the name of the png in case it has multiple orientation.
                            
                            // png_path.push(path.clone() + file_name);
                        } else {
                            godot_warn!{"found unknow: {}", &file_name}
                        }
                        file_name = dir.get_next().to_string();
                    }
                    dir.list_dir_end();
                } else {
                    godot_error!("Could not list dir in: {}.", path);
                }
            } else {
                // TODO: Missing mod cancel loading world.
                godot_error!("Could not open folder: {}.", path);
            }
        }

        Self {
            monsters: todo!(),
            sprites_paths: todo!(),
            mod_order: todo!(),
        }
    }
}

/// Represent a mod with its version.
#[derive(Hash, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModInfo {
    pub name: String,
    pub version: String,
}
impl Default for ModInfo {
    fn default() -> Self {
        Self {
            name: APP_NAME.to_string(),
            version: String::new(),
        }
    }
}
impl ModInfo {
    /// Return a path to the mod folder. Does not guaranty that the mod folder exist.
    /// # Examples
    /// MonInfo {name: "A super mod", version: "10.10 extra plus"}
    ///
    /// user://mods/A super mod/10_10 extra plus/
    pub fn get_path(&self) -> String {
        if self.name == APP_NAME {
            // The base game is in res://base/
            BASE_GAME_PATH.to_string()
        } else if !self.version.is_empty() {
            // A regular mod is in user://mods/mod_name/version_name/
            format!("{}{}/{}/", MODS_PATH, self.name, self.version).replace(".", "_")
        } else {
            // A regular mod without version is in user://mods/mod_name/
            format!("{}{}/", MODS_PATH, self.name).replace(".", "_")
        }
    }
}

/// Load saved mod_order or return a default one with just APP_NAME.
fn load_mod_order(world_path: &str) -> Vec<ModInfo> {
    let mut mod_order = Vec::new();

    let file = gdnative::api::File::new();

    // Try to open saved mod_order.
    let mod_order_path = format!("{}mod_order", &world_path);
    if file.open(&mod_order_path, gdnative::api::File::READ).is_ok() {
        let mut line = file.get_csv_line(",");
        while !line.is_empty() {
            {
                let line_read = line.read();

                // Get mod name.
                if let Some(mod_name) = line_read.first() {
                    if mod_name.is_empty() {
                        break;
                    }

                    // Get mod version.
                    let mut mod_version = String::new();
                    if let Some(new_mod_version) = line_read.get(1) {
                        mod_version = new_mod_version.to_string();
                    }

                    mod_order.push(ModInfo {
                        name: mod_name.to_string(),
                        version: mod_version,
                    });
                } else {
                    break;
                }
            }
            line = file.get_csv_line(",");
        }
    } else {
        godot_warn!("Could not open {}.", mod_order_path);
    }
    file.close();

    // // Check that each mod exist.
    // let dir = Directory::new();
    // mod_order.drain_filter(|mod_name| {
    //     let mod_path = get_mod_path(mod_name);
    //     if dir.dir_exists(&mod_path) {
    //         return false;
    //     }
    //     godot_warn!("Could not find {} at {}.", mod_name, &mod_path);
    //     true
    // });

    // Make sure we a have at least one mod.
    if mod_order.is_empty() {
        mod_order.push(ModInfo::default());
    }

    godot_print!("Loaded mod_order: {:?}.", &mod_order);

    mod_order
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
