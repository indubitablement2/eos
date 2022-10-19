#![feature(drain_filter)]
#![feature(hash_drain_filter)]
#![feature(map_try_insert)]
#![feature(is_some_and)]
#![feature(variant_count)]

extern crate nalgebra as na;

mod client;
mod client_battlescape;
mod constants;
mod godot_client_config;
mod godot_logger;
pub mod shared;
mod time_manager;
mod util;

// mod input_handler;
// mod metasacpe_manager;
// pub mod metascape_runner;
// mod battlescape;

static LOGGER: godot_logger::GodotLogger = godot_logger::GodotLogger;

// Function that registers all exposed classes to Godot
fn init(handle: gdnative::prelude::InitHandle) {
    // Init GodotLogger.
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .expect("can not start logger");

    handle.add_class::<client::Client>();
    handle.add_class::<client_battlescape::ClientBattlescape>();
}

// Macros that create the entry-points of the dynamic library.
gdnative::prelude::godot_init!(init);
