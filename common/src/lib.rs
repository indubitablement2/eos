#![feature(test)]
#![feature(duration_constants)]
#![feature(slice_split_at_unchecked)]
#![feature(slice_as_chunks)]
#![feature(hash_drain_filter)]
#![feature(split_array)]
#![feature(array_chunks)]

#[macro_use]
extern crate log;

pub mod factions;
pub mod idx;
pub mod net;
pub mod orbit;
pub mod reputation;
pub mod ships;
pub mod systems;
pub mod time;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long between each Battlescape/Metascape tick.
pub const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/// The range used for relative compressed metascape position sent by the server.
pub const METASCAPE_RANGE: f32 = 512.0;