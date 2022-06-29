#![feature(drain_filter)]
#![feature(hash_drain_filter)]
#![feature(map_try_insert)]
#![feature(is_some_with)]

#[macro_use]
extern crate log;

extern crate nalgebra as na;

mod client;
mod configs;
mod connection_manager;
mod constants;
mod godot_logger;
mod input_handler;
mod metascape;
mod time_manager;
mod util;

static LOGGER: godot_logger::GodotLogger = godot_logger::GodotLogger;

// Function that registers all exposed classes to Godot
fn init(handle: gdnative::prelude::InitHandle) {
    // Init GodotLogger.
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .expect("can not start logger");

    // TODO: Init data.
    common::data::Data::default().init();

    handle.add_class::<client::Client>();
}

// Macros that create the entry-points of the dynamic library.
gdnative::prelude::godot_init!(init);
