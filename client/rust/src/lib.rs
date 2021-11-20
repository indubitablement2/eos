#![feature(drain_filter)]
#[macro_use]
extern crate log;

pub mod constants;
pub mod def;

pub mod ecs;
pub mod ecs_components;
pub mod ecs_input;
pub mod ecs_resources;
pub mod ecs_systems;

pub mod ecs_render_pipeline;
pub mod yaml_components;

mod client;
mod game;
mod godot_logger;

use gdnative::prelude::{godot_init, InitHandle};
use godot_logger::GodotLogger;

static LOGGER: GodotLogger = GodotLogger;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    // Init GodotLogger.
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .expect("can not start logger");

    handle.add_class::<game::Game>();
}

// Macros that create the entry-points of the dynamic library.
godot_init!(init);
