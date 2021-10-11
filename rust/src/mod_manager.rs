use crate::constants::*;
use gdnative::api::*;
use gdnative::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_builder)]
pub struct ModManager {
    mod_config: ModConfig,
    game_def: Option<GameDef>,
    /// The atlas texture where all sprites are packed.
    atlas_texture: Option<Ref<TextureArray>>,
}

#[methods]
impl ModManager {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node) -> Self {
        Self {
            mod_config: ModConfig::default(),
            atlas_texture: None,
            game_def: None
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node) {
            // let data = file.get_var(false).to_byte_array();
            // let data_read = data.read();
            // if let Ok(new_mod_config) = bincode::deserialize::<ModConfig>(&data_read) {
            //     self.mod_config = new_mod_config;
            // } else {
            //     godot_error!("Could not deserialize mod config.");
            // }
    }

    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node) {
        // Save mod config.
        let file = File::new();
        if file.open(MOD_CONFIG_FILE_PATH, File::WRITE).is_ok() {
            if let Ok(data) = bincode::serialize(&self.mod_config) {
                file.store_var(TypedArray::from_vec(data), false);
            }
        }
        file.close();
    }
}

#[derive(Serialize, Deserialize)]
struct ModConfig {
    /// Order in which mods are loaded. Only the first *num_enabled_mod* are relevant.
    pub mod_order: Vec<String>,
    /// Number of enabled mods.
    pub num_enabled_mod: u32,
}
impl Default for ModConfig {
    fn default() -> Self {
        Self {
            mod_order: Vec::new(),
            num_enabled_mod: 0,
        }
    }
}

pub struct GameDef {

}
impl GameDef {

}