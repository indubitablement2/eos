mod battlescape;
mod central_server;
mod connection;
mod database;
mod instance_server;
mod logger;
mod runner;

use ahash::{AHashMap, AHashSet, RandomState};
use battlescape::Battlescape;
use connection::*;
use dashmap::DashMap;
use indexmap::IndexMap;
use parking_lot::Mutex;
use rand::prelude::*;
use rapier2d::na::{self, Isometry2, Vector2};
use serde::{Deserialize, Serialize};
use std::sync::atomic;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::time::{Duration, Instant};
use std::{
    f32::consts::{FRAC_PI_2, PI, TAU},
    net::SocketAddr,
};

const TARGET_DT_DURATION: Duration = Duration::from_millis(50);
pub const TARGET_DT: f32 = 0.05;

/// Address for the instance servers to connect to the central server.
pub const CENTRAL_ADDR_INSTANCE: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
    std::net::Ipv6Addr::LOCALHOST,
    12461,
    0,
    0,
));
/// Address for the clients to connect to the central server.
pub const CENTRAL_ADDR_CLIENT: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
    std::net::Ipv6Addr::LOCALHOST,
    8461,
    0,
    0,
));

// TODO: Use non-zero IDs.
// TODO: Store receiver channel (db -> battlescape) in battlescape

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BattlescapeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

fn main() {
    logger::Logger::init();

    // EntityData::set_data(vec![EntityData::default()]);

    let (mut runner, new_battlescape_sender) = runner::BattlescapesRunner::new(2);
    database::Database::start(new_battlescape_sender);

    let mut previous_step = Instant::now();
    loop {
        if let Some(remaining) = TARGET_DT_DURATION.checked_sub(previous_step.elapsed()) {
            std::thread::sleep(remaining);
        }

        let now = Instant::now();
        let delta = (now - previous_step).as_secs_f32().min(TARGET_DT * 2.0);
        previous_step = now;
        runner.step(delta);
    }
}
