mod battlescape;
mod central_server;
mod connection;
mod instance_server;
mod logger;

use ahash::{AHashMap, AHashSet, RandomState};
use connection::*;
// use battlescape::entity::{EntityData, EntityDataId};
use dashmap::DashMap;
use indexmap::IndexMap;
use parking_lot::Mutex;
use rand::prelude::*;
use rapier2d::na::{self, Isometry2, Vector2};
use serde::{Deserialize, Serialize};
use std::sync::atomic;
use std::{
    f32::consts::{FRAC_PI_2, PI, TAU},
    net::SocketAddr,
};

const PRIVATE_KEY: u64 = const_random::const_random!(u64);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BattlescapeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

#[tokio::main]
async fn main() {
    logger::Logger::init();

    log::debug!("{:x}", PRIVATE_KEY);

    // EntityData::set_data(vec![EntityData::default()]);

    let mut central = false;
    let mut instance = false;
    for arg in std::env::args() {
        if &arg == "instance" {
            instance = true;
        } else if &arg == "central" {
            central = true;
        }
    }
    if !central && !instance {
        log::warn!("No arguments specified, defaulting to 'instance' and 'central'");
        central = true;
        instance = true;
    }

    if central && instance {
        tokio::spawn(instance_server::_start());
        central_server::_start().await;
    } else if central {
        central_server::_start().await;
    } else if instance {
        instance_server::_start().await;
    }
}
