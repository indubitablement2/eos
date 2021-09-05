pub mod chunk_generator;

pub mod ecs_resources;
pub mod ecs_systems;
pub mod ecs_schedue;
pub mod ecs_components;

mod chunk;

use gdnative::prelude::{godot_init, InitHandle};

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<chunk::Chunk>();
}

// Macros that create the entry-points of the dynamic library.
godot_init!(init);
