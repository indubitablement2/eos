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
mod metascape_system;
pub mod packets;

// pub const SIZE_SMALL_FLEET: f32 = 0.1;
// pub const SIZE_GAUGING_NORMAL_PLANET: f32 = 1.0;
// pub const SIZE_NORMAL_STAR: f32 = 4.0;
