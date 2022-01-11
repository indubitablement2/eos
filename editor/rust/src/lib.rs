#![feature(drain_filter)]
#![feature(hash_drain_filter)]

mod editor;
mod generation;
mod util;

use gdnative::prelude::{godot_init, InitHandle};

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<editor::Editor>();
}

// Macros that create the entry-points of the dynamic library.
godot_init!(init);
