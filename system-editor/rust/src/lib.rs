#![feature(drain_filter)]
#![feature(hash_drain_filter)]
#[macro_use]
extern crate log;

mod system_editor;
mod godot_logger;
mod util;
mod generation;
mod generation_mask;
mod systems_merge;

use gdnative::prelude::{godot_init, InitHandle};
use godot_logger::GodotLogger;

static LOGGER: GodotLogger = GodotLogger;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    // Init GodotLogger.
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .expect("can not start logger");

    handle.add_class::<system_editor::SystemEditor>();
}

// Macros that create the entry-points of the dynamic library.
godot_init!(init);
