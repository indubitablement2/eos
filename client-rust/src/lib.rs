#![feature(drain_filter)]
#![feature(hash_drain_filter)]
#![feature(map_try_insert)]
#![feature(is_some_and)]
#![feature(variant_count)]

extern crate nalgebra as na;

mod client;
mod client_battlescape;
mod constants;
pub mod draw;
mod godot_logger;
mod time_manager;
mod util;

// Function that registers all exposed classes to Godot
fn init(handle: gdnative::prelude::InitHandle) {
    // Init GodotLogger.
    godot_logger::GodotLogger::init();

    handle.add_class::<client::Client>();
}

// Macros that create the entry-points of the dynamic library.
gdnative::prelude::godot_init!(init);
