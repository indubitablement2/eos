// mod battlescape;
mod connection;
mod database;
mod godot_encoding;
mod ids;
// mod instance_server;
mod interval;
mod logger;

use ahash::{AHashMap, AHashSet, RandomState};
use connection::*;
use crossbeam_channel::{bounded, unbounded, Receiver, RecvError, Sender, TryRecvError};
use ids::*;
use indexmap::IndexMap;
use rand::prelude::*;
use rapier2d::na::{self, Isometry2, Vector2};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use std::{
    f32::consts::{FRAC_PI_2, PI, TAU},
    net::SocketAddr,
};

// TODO: Keep track of what data client has and send as needed instead of waiting for query

// TODO: database (rmp, bincode)
// // TODO: instance <-> database (bincode)
// TODO: replay (rmp)
// TODO: instance <-> client (godot?)

const PRIVATE_KEY: u64 = const_random::const_random!(u64);

static _TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
fn tokio() -> &'static tokio::runtime::Runtime {
    _TOKIO_RUNTIME.get().unwrap()
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

    // EntityData::set_data(vec![EntityData::default()]);

    let mut database = false;
    let mut instance = false;

    for arg in std::env::args() {
        if &arg == "instance" {
            instance = true;
        } else if &arg == "database" {
            database = true;
        } else if let Some(addr) = arg
            .strip_prefix("database_addr=")
            .and_then(|arg| arg.parse().ok())
        {
            _DATABASE_ADDR.set(addr);
        } else if let Some(addr) = arg
            .strip_prefix("instance_addr=")
            .and_then(|arg| arg.parse().ok())
        {
            _INSTANCE_ADDR.set(addr);
        }
    }
    if !database && !instance {
        log::warn!("No arguments specified, defaulting to 'database' and 'instance'");
        database = true;
        instance = true;
    }

    if database {
        log::info!("Database address: {}", database_addr());
    }
    if instance {
        log::info!("Instance address: {}", instance_addr());
    }

    if database && instance {
        std::thread::spawn(|| {});
        database::_start();
    } else if database {
        database::_start();
    } else if instance {
        // TODO
    }
}
