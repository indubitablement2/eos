mod connection;
mod database;
mod godot_encoding;
mod ids;
mod interval;
mod logger;

use ahash::{AHashMap, AHashSet, RandomState};
use connection::Packet;
use ids::*;
use indexmap::IndexMap;
use parking_lot::Mutex;
use rand::prelude::*;
use rapier2d::na::{self, Isometry2, Vector2};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, sync_channel, Receiver, RecvError, Sender, SyncSender};
use std::time::{Duration, Instant};
use std::{
    f32::consts::{FRAC_PI_2, PI, TAU},
    net::SocketAddr,
};

const PRIVATE_KEY: u64 = const_random::const_random!(u64);

fn main() {
    logger::Logger::init();

    log::debug!("{}", PRIVATE_KEY);

    // EntityData::set_data(vec![EntityData::default()]);

    let mut database = false;
    let mut instance = false;
    let mut database_addr = SocketAddr::V6(std::net::SocketAddrV6::new(
        std::net::Ipv6Addr::LOCALHOST,
        17384,
        0,
        0,
    ));
    for arg in std::env::args() {
        if &arg == "instance" {
            instance = true;
        } else if &arg == "database" {
            database = true;
        } else if let Ok(addr) = arg.parse() {
            database_addr = addr;
        }
    }
    if !database && !instance {
        log::warn!("No arguments specified, defaulting to 'instance' and 'central'");
        database = true;
        instance = true;
    }

    log::info!("Database address: {}", database_addr);

    if database && instance {
        std::thread::spawn(|| {});
        database::_start(database_addr);
    } else if database {
        database::_start(database_addr);
    } else if instance {
        // TODO
    }
}
