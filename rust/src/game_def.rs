use crate::constants::*;
use crate::yaml_components::*;
use gdnative::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    hash::{Hash, Hasher},
};

#[derive(Serialize, Deserialize)]
pub struct GameDef {
    pub corrupted: bool,
    pub entities_bundles: Vec<Vec<EcsComponents>>,
    /// The individual sprite location. We keep this to quickly remake the sprite array in case it is deleted.
    pub sprites_paths: Vec<String>,
    /// The order in which mods are loaded.
    pub mod_order: Vec<ModInfo>,
}
impl GameDef {
    /// Load a cached game_def or create a new one. Return true if it is corrupted.
    pub fn load(world_path: &str, try_load_from_cache: bool, save_if_new: bool) -> Self {
        let file = gdnative::api::File::new();

        // Get mod_order.
        let mod_order = load_mod_order(world_path);

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

                        debug_assert!(!game_def.corrupted);
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

        // Create a new game_def.
        let game_def = GameDef::new(mod_order);

        // Save game_def for faster load time.
        if save_if_new && !game_def.corrupted {
            if file.open(&game_def_path, gdnative::api::File::WRITE).is_ok() {
                if let Ok(data) = bincode::serialize(&game_def) {
                    let num_byte = i64::try_from(data.len()).unwrap_or_default();
                    file.store_64(num_byte);
                    file.store_buffer(TypedArray::from_vec(data));
                    godot_print!("Cached GameDef.");
                } else {
                    godot_error!("Could not serialize game_def to cache it.");
                }
            } else {
                godot_error!("Could not open {} to cahce game_def.", &game_def_path);
            }
            file.close();
        } else {
            godot_print!("Not caching GameDef.");
        }

        game_def
    }

    /// This does a best effort to compile all mods. Return a potentialy corrupted GameDef if any error where encountered.
    fn new(mod_order: Vec<ModInfo>) -> Self {
        let dir = gdnative::api::Directory::new();
        let file = gdnative::api::File::new();

        let mut corrupted = false;

        // Path are relative to mod folder.
        // Path: user://mods/my mod/ver_1_002_123/asset/mon.yaml would result in ("asset/mon.yaml", mod_id).

        let mut folder_path_to_parse = Vec::with_capacity(256);

        // All the yaml file that were found ordered by first to last mod to load.
        let mut yaml_paths = Vec::with_capacity(1024);

        // Start by adding all mod folders.
        for mod_id in 0..mod_order.len() {
            folder_path_to_parse.push(("".to_string(), mod_id));
        }

        // Find all yaml files.
        while let Some((path, mod_id)) = folder_path_to_parse.pop() {
            if dir.open(format!("{}{}", mod_order[mod_id].get_path(), &path)).is_ok() {
                if dir.list_dir_begin(true, true).is_ok() {
                    let mut file_name = dir.get_next().to_string();
                    while !file_name.is_empty() {
                        if dir.current_is_dir() {
                            godot_print!("found dir: {}", &file_name);
                            folder_path_to_parse.push((format!("{}{}/", &path, &file_name), mod_id));
                        } else if file_name.ends_with(".yaml") {
                            godot_print!("found yaml: {}", &file_name);
                            yaml_paths.push((format!("{}{}", &path, &file_name), mod_id));
                        }
                        file_name = dir.get_next().to_string();
                    }
                    dir.list_dir_end();
                } else {
                    corrupted = true;
                    godot_error!("Could not list dir for {}. Ignoring this folder.", path);
                }
            } else {
                corrupted = true;
                godot_error!("Could not open {}. Ignoring this folder.", path);
            }
        }

        // Parse all yaml into a vector of vector of YamlComponents.
        let mut list_yaml_components = Vec::with_capacity(yaml_paths.len());
        for (yaml_relative_path, mod_id) in yaml_paths.into_iter() {
            let abs_path = format!("{}{}", mod_order[mod_id].get_path(), &yaml_relative_path);
            if file.open(&abs_path, gdnative::api::File::READ).is_ok() {
                if let Ok(yaml_components) = serde_yaml::from_str::<Vec<Vec<YamlComponents>>>(&file.get_as_text().to_string()) {
                    list_yaml_components.push(yaml_components);
                } else {
                    // Ignore this file as it can not be deserialized.
                    corrupted = true;
                    godot_error!("Could not deserialize Yaml {}. Ignoring file.", abs_path);
                }
            } else {
                corrupted = true;
                godot_error!("Could not open Yaml {}. Ignoring file.", abs_path);
            }
        }

        // Parse YamlComponents to EcsComponents.
        let yaml_parse_result = YamlParseResult::parse_yaml_components(list_yaml_components);
        Self {
            corrupted: corrupted && yaml_parse_result.corrupted,
            entities_bundles: yaml_parse_result.entity_bundles,
            sprites_paths: yaml_parse_result.sprites,
            mod_order,
        }
    }
}

/// Represent a mod with its version.
#[derive(Hash, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
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
        godot_error!("Could not open {}.", mod_order_path);
    }
    file.close();

    // Make sure we a have at least one mod.
    if mod_order.is_empty() {
        mod_order.push(ModInfo::default());
    }

    godot_print!("Loaded mod_order: {:?}.", &mod_order);

    mod_order
}
