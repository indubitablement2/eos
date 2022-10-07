#![feature(test)]
#![feature(duration_constants)]
#![feature(slice_split_at_unchecked)]
#![feature(slice_as_chunks)]
#![feature(hash_drain_filter)]
#![feature(split_array)]
#![feature(array_chunks)]
#![feature(io_error_other)]
#![feature(duration_consts_float)]

pub mod data;
pub mod fleet;
pub mod idx;
pub mod net;
pub mod orbit;
pub mod rand_vector;
pub mod reputation;
pub mod system;
pub mod command;

extern crate nalgebra as na;

pub use data::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The expected real world time duration of a `Metascape` tick.
/// 
/// 10 ups
pub const METASCAPE_TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(100);
pub const METASCAPE_TICK_DURATION_SEC: f32 = METASCAPE_TICK_DURATION.as_secs_f32();
pub const METASCAPE_TICK_DURATION_MIL: u32 = METASCAPE_TICK_DURATION.as_millis() as u32;

/// The expected real world time duration of a `Battlescape` tick.
/// 
/// 20 ups
pub const BATTLESCAPE_TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(50);
pub const BATTLESCAPE_TICK_DURATION_SEC: f32 = BATTLESCAPE_TICK_DURATION.as_secs_f32();
pub const BATTLESCAPE_TICK_DURATION_MIL: u32 = BATTLESCAPE_TICK_DURATION.as_millis() as u32;
