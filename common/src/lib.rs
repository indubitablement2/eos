#![feature(test)]
#![feature(duration_constants)]
#![feature(slice_split_at_unchecked)]
#![feature(slice_as_chunks)]
#![feature(hash_drain_filter)]
#![feature(split_array)]
#![feature(array_chunks)]

#[macro_use]
extern crate log;

pub mod data;
pub mod factions;
pub mod fleet;
pub mod idx;
pub mod net;
pub mod orbit;
pub mod reputation;
pub mod system;
pub mod time;

pub use data::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long between each Battlescape/Metascape tick.
pub const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
