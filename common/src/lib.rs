#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]

#[macro_use]
extern crate log;

mod collision;
mod connection_manager;
pub mod generation;
pub mod metascape;
pub mod packets;
