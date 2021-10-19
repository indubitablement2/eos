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
    pub entities_bundles: Vec<Vec<EcsComponents>>,
    /// The individual sprite location.
    pub sprites_paths: Vec<String>,
    /// The order in which mods are loaded.
    pub mod_order: Vec<ModInfo>,
}
impl GameDef {
    /// Load a cached game_def or create a new one.
    pub fn load(world_path: &str, try_load_from_cache: bool, save_if_new: bool) -> Result<Self, GameDefLoadError> {
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
                        return Ok(game_def);
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
        let game_def_result = GameDef::new(mod_order);

        // Save game_def for faster load time.
        if save_if_new {
            if let Ok(game_def) = &game_def_result {
                if file.open(&game_def_path, gdnative::api::File::WRITE).is_ok() {
                    if let Ok(data) = bincode::serialize(game_def) {
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
        }

        game_def_result
    }

    fn new(mod_order: Vec<ModInfo>) -> Result<Self, GameDefLoadError> {
        let dir = gdnative::api::Directory::new();
        let file = gdnative::api::File::new();

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
                    return Err(GameDefLoadError::CouldNotListDir(path));
                }
            } else {
                return Err(GameDefLoadError::MissingMod(path));
            }
        }

        // Parse all yaml into a vector of vector of YamlComponents.
        let mut list_yaml_components = Vec::with_capacity(yaml_paths.len() * 20);
        for (yaml_relative_path, mod_id) in yaml_paths.into_iter() {
            let abs_path = format!("{}{}", mod_order[mod_id].get_path(), &yaml_relative_path);
            if file.open(&abs_path, gdnative::api::File::READ).is_ok() {
                if let Ok(mut yaml_components) = serde_yaml::from_str::<Vec<Vec<YamlComponents>>>(&file.get_as_text().to_string())
                {
                    list_yaml_components.push(yaml_components);
                } else {
                    return Err(GameDefLoadError::CouldNotDeserializeYaml(abs_path));
                }
            } else {
                return Err(GameDefLoadError::CouldNotOpenYaml(abs_path));
            }
        }

        // Parse YamlComponents to EcsComponents.
        match parse_yaml_components(&list_yaml_components) {
            Ok((entities_bundles, sprites_paths)) => Ok(Self {
                entities_bundles,
                sprites_paths,
                mod_order,
            }),
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameDefLoadError {
    MissingMod(String),
    CouldNotListDir(String),
    CouldNotOpenYaml(String),
    CouldNotDeserializeYaml(String),
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
