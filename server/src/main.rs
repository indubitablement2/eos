mod battlescape;
mod connection;
mod data;
mod database;
mod godot_encoding;
mod ids;
mod instance;
mod interval;
mod logger;

use ahash::{AHashMap, AHashSet, RandomState};
use anyhow::Context;
use connection::*;
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use data::data;
use database::*;
use ids::*;
use indexmap::IndexMap;
use rand::prelude::*;
use rapier2d::na::{self, Isometry2, Point2, Vector2};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

// TODO: add feature for database/instance
// TODO: Mini app which compile and relauches instance and database if they exit
// TODO: Private key taken from file
// TODO: Websocket encryption

// TODO: Database:
// Do not store encoded value in database
// Check invariants on startup (armor cell size, username -> client, all battlescapes from data exist)
// // Add global time tracking
// // Create battlescape cmd
// // Balance battlescapes
// // move ship to battlescape cmd
// // notify instance ship changes and send to client (subscribtion based)

// TODO: Instance:
// Keep track of what data client has and send as needed instead of waiting for query
// add ships to intermitent battlescape save

// TODO: Battlescape:
// figure out how to handle collisions
// shield

// TODO: Client:
// add c++
// impl binary decoder/encoder
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

fn bin_encode(data: impl Serialize) -> Vec<u8> {
    postcard::to_allocvec(&data).unwrap()
}
fn bin_decode<'a, T: Deserialize<'a>>(data: &'a [u8]) -> anyhow::Result<T> {
    Ok(postcard::from_bytes(data)?)
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

    data::load_data();

    let mut database = false;
    let mut instance = false;

    for arg in std::env::args() {
        log::info!("Arg: {}", arg);

        if &arg == "instance" {
            instance = true;
        } else if &arg == "database" {
            database = true;
        }
    }
    if !database && !instance {
        log::warn!("No arguments specified, defaulting to 'database' and 'instance'");
        database = true;
        instance = true;
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
