#![feature(duration_consts_float)]

mod data;
mod id;
mod logger;
mod server;
mod small_id_dispenser;
mod system;

use ahash::{AHashMap, AHashSet};
use anyhow::{anyhow, Result};
use log::{debug, error, info, trace, warn};
use rapier2d::na::{self, Isometry2, Point2, Vector2};
use serde::{Deserialize, Serialize};
// use smallvec::{smallvec, SmallVec};
// use std::f32::consts::{FRAC_PI_2, PI, TAU};

use data::*;
use id::*;
use small_id_dispenser::*;
use system::System;

pub const DELTA: std::time::Duration = std::time::Duration::from_millis(50);
pub const DT: f32 = DELTA.as_secs_f32();

fn main() {
    logger::Logger::init();

    let mut server = server::Server::load().expect("server should load");

    let mut buf = String::new();
    let stdin = std::io::stdin();
    loop {
        let now = std::time::Instant::now();

        stdin.read_line(&mut buf).unwrap();
        if buf.trim() == "exit" {
            break;
        }
        buf.clear();

        server.step();

        let elapsed = now.elapsed();
        if let Some(remaining) = DELTA.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        } else {
            warn!("Server is running behind!");
        }
    }
}
