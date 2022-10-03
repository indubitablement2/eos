#![feature(test)]
#![feature(duration_constants)]
#![feature(slice_split_at_unchecked)]
#![feature(slice_as_chunks)]
#![feature(hash_drain_filter)]
#![feature(split_array)]
#![feature(array_chunks)]
#![feature(io_error_other)]

pub mod data;
pub mod fleet;
pub mod idx;
pub mod net;
pub mod orbit;
pub mod rand_vector;
pub mod reputation;
pub mod system;
pub mod timef;

extern crate nalgebra as na;

pub use data::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// How long between each Battlescape/Metascape tick.
///
/// The duration of a tick.
#[deprecated]
pub const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/// The real world time duration of a tick.
pub const TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(100);

/// Minimun number of ticks before reusing small idx.
pub const SMALL_ID_RECYCLE_DELAY: u32 = 100;
