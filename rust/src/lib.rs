pub mod battlescape;
pub mod battlescape_components;
pub mod battlescape_input;
pub mod battlescape_resources;
pub mod battlescape_schedue;
pub mod battlescape_systems;
mod game;

use gdnative::prelude::{godot_init, InitHandle};

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<game::Game>();
}

// Macros that create the entry-points of the dynamic library.
godot_init!(init);
