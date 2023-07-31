#![feature(duration_consts_float)]

mod client_connection;
mod data;
mod id;
mod logger;
mod master_server;
mod simulation_server;
mod small_id_dispenser;
mod system;

use ahash::{AHashMap, AHashSet};
use anyhow::{anyhow, Context, Result};
use bytes::BufMut;
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

    data::Data::initialize();

    let mut master_started = false;
    let mut sim_started = false;

    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut buf = String::new();
    let stdin = std::io::stdin();
    while let Ok(_) = stdin.read_line(&mut buf) {
        let cmd = buf.trim().trim_end_matches('\n').trim();

        if cmd == "exit" {
            break;
        } else if cmd.starts_with("start") {
            if cmd.contains("master") || cmd.contains("both") && !master_started {
                info!("Starting master server");
                master_started = true;
                rt.spawn(master_server::main());
            }
            if cmd.contains("runner") || cmd.contains("both") && !sim_started {
                info!("Starting runner server");
                sim_started = true;
                // rt.spawn(simulation_server::main());
            }
        }

        buf.clear();
    }
}
