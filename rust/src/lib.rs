pub mod battlescape_components;
pub mod battlescape_resources;
pub mod battlescape_schedue;
pub mod battlescape_systems;

pub mod battlescape_input;
mod godot_ecs;
pub mod battlescape;

use gdnative::prelude::{godot_init, InitHandle};

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<godot_ecs::GodotEcs>();
}

// Macros that create the entry-points of the dynamic library.
godot_init!(init);
