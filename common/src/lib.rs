#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(slice_split_at_unchecked)]
#![feature(option_result_unwrap_unchecked)]

#[macro_use]
extern crate log;

// mod collision_old_rapier;
// mod collision_old_grid;
mod collision;
mod connection_manager;
pub mod generation;
pub mod metascape;
pub mod packets;
