use crate::constants::*;
use gdnative::api::*;
use gdnative::prelude::*;
use serde::{Deserialize, Serialize};
// use bincode::*;

/// Store most config.
#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_builder)]
pub struct ConfigManager {
    pub config: Config,
}

#[methods]
impl ConfigManager {
    // Register the builder for methods, properties and/or signals.
    fn register_builder(_builder: &ClassBuilder<Self>) {}

    /// The "constructor" of the class.
    fn new(_owner: &Node) -> Self {
        Self {
            config: Config::default()
        }
    }

    #[export]
    unsafe fn _ready(&mut self, _owner: &Node) {
        
    }

    #[export]
    unsafe fn _exit_tree(&mut self, _owner: &Node) {
        // Save config to file.
        let file = File::new();
        match file.open(CONFIG_FILE_PATH, File::WRITE) {
            Ok(_) => {

            }
            Err(err) => {
                godot_error!("Can not save config: {:?}.", err)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    audio_config: AudioConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self { 
            audio_config: Default::default(),
        }
    }
}

/// Contain volume in linear energy.
#[derive(Serialize, Deserialize, Clone, Copy)]
struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_ui_volume: f32,
    pub sfx_2d_volume: f32,
}
impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_ui_volume: 1.0,
            sfx_2d_volume: 1.0,
        }
    }
}
