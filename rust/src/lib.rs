pub mod constants;
pub mod render_util;

pub mod ecs;
pub mod ecs_components;
pub mod ecs_input;
pub mod ecs_resources;
pub mod ecs_systems;

mod godot_game;
pub mod singleton_config_manager;
mod singleton_mod_manager;

use gdnative::prelude::{godot_init, InitHandle};

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<godot_game::Game>();
    handle.add_class::<singleton_mod_manager::ModManager>();
}

// Macros that create the entry-points of the dynamic library.
godot_init!(init);
