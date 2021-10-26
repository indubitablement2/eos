#![feature(drain_filter)]

pub mod constants;
pub mod def;

pub mod ecs;
pub mod ecs_components;
pub mod ecs_input;
pub mod ecs_resources;
pub mod ecs_systems;

pub mod ecs_render_pipeline;
pub mod yaml_components;

mod world;

use gdnative::prelude::{godot_init, InitHandle};

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<world::World>();
}

// Macros that create the entry-points of the dynamic library.
godot_init!(init);
