mod battlescape;
mod connection;
mod database;
mod godot_encoding;
mod ids;
mod instance;
mod interval;
mod logger;
mod runner;

use ahash::{AHashMap, AHashSet, RandomState};
use anyhow::Context;
use connection::*;
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use ids::*;
use indexmap::IndexMap;
use rand::prelude::*;
use rapier2d::na::{self, Isometry2, Point2, Vector2};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

// modular connection

// TODO: add feature for database/instance
// TODO: Replace bincode for msgpack
// TODO: instance-database communication
// TODO: Static battlescape server address

// TODO: Database:
// Create battlescape cmd
// Balance battlescapes
// move ship to battlescape cmd
// notify instance ship changes and send to client (subscribtion based)

// TODO: Instance:
// Keep track of what data client has and send as needed instead of waiting for query
// add ships to intermitent battlescape save

// TODO: Battlescape:
// figure out how to handle collisions
// shield

// TODO: Client:
// add c++
// impl bincode decoder/encoder
// add packet base class and one child for each packet type

const _PRIVATE_KEY_FALLBACK: [u8; 64] = const_random::const_random!([u8; 64]);
static _PRIVATE_KEY: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
fn private_key() -> &'static [u8] {
    _PRIVATE_KEY
        .get()
        .map(Vec::as_slice)
        .unwrap_or(&_PRIVATE_KEY_FALLBACK)
}

static _TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
fn tokio() -> &'static tokio::runtime::Runtime {
    _TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

static _DATABASE_ADDR: std::sync::OnceLock<SocketAddr> = std::sync::OnceLock::new();
fn database_addr() -> SocketAddr {
    _DATABASE_ADDR
        .get()
        .copied()
        .unwrap_or(SocketAddr::V6(std::net::SocketAddrV6::new(
            std::net::Ipv6Addr::LOCALHOST,
            17384,
            0,
            0,
        )))
}

static _INSTANCE_ADDR: std::sync::OnceLock<SocketAddr> = std::sync::OnceLock::new();
fn instance_addr() -> SocketAddr {
    _INSTANCE_ADDR
        .get()
        .copied()
        .unwrap_or(SocketAddr::V6(std::net::SocketAddrV6::new(
            std::net::Ipv6Addr::LOCALHOST,
            17385,
            0,
            0,
        )))
}

fn bincode_encode(data: impl Serialize) -> Vec<u8> {
    bincode::Options::serialize(bincode::DefaultOptions::new(), &data).unwrap()
}
fn bincode_decode<'a, T: Deserialize<'a>>(data: &'a [u8]) -> anyhow::Result<T> {
    Ok(bincode::Options::deserialize(
        bincode::DefaultOptions::new(),
        data,
    )?)
}

fn main() {
    logger::Logger::init();

    _TOKIO_RUNTIME
        .set(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .unwrap();

    battlescape::entity::_load_data();

    let mut database = false;
    let mut instance = false;

    for arg in std::env::args() {
        log::info!("Arg: {}", arg);

        if &arg == "instance" {
            instance = true;
        } else if &arg == "database" {
            database = true;
        } else if let Some(addr) = arg
            .strip_prefix("database_addr=")
            .and_then(|arg| arg.parse().ok())
        {
            let _ = _DATABASE_ADDR.set(addr);
        } else if let Some(addr) = arg
            .strip_prefix("instance_addr=")
            .and_then(|arg| arg.parse().ok())
        {
            let _ = _INSTANCE_ADDR.set(addr);
        } else if let Some(key) = arg.strip_prefix("key=") {
            let _ = _PRIVATE_KEY.set(key.as_bytes().to_vec());
        }
    }
    if !database && !instance {
        log::warn!("No arguments specified, defaulting to 'database' and 'instance'");
        database = true;
        instance = true;
    }

    log::info!("Database address: {}", database_addr());
    if instance {
        log::info!("Instance address: {}", instance_addr());
    }

    if database && instance {
        std::thread::spawn(|| database::_start());
        instance::_start();
    } else if database {
        database::_start();
    } else if instance {
        instance::_start();
    }
}
