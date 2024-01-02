mod connection;
mod data;
mod database;
mod godot_encoding;
mod ids;
mod instance;
mod interval;
mod logger;
mod simulation;
mod util;

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
use std::f32::consts::TAU;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

// // TODO: add feature for database/instance
// // TODO: Mini app which compile and relauches instance and database if they exit
// // TODO: Private key taken from file
// TODO: Websocket encryption

// TODO: Database:
// Add starting ship database resquest (if no ship)
// keep track of logged-in client
// only send fleet update to client when something changes
// only store hashed password
// // Check invariants on startup (armor cell size, username -> client, all simulations from data exist)
// // Do not store encoded value in database
// // Add global time tracking
// // Create simulation cmd
// // Balance simulations
// // move ship to simulation cmd
// // notify instance ship changes and send to client (subscribtion based)

// TODO: Instance:
// // remove client outbound (only simulation has any)
// add fast way for client to change simulation on same instance without reconnect
// simulation packets shouldn't need to pass through instance

// TODO: Simulation:
// add ships to intermitent simulation save
// Keep track of what data client has and send as needed instead of waiting for query
// Add entity detection and detector range
// remove uneeded derives
// // has its own id range for entity/ship based on sim id
// Variable dt (for "sleeping" simulations)
// change collision groups to an enum
// projectile: simple vec + manual query. Updated client side
// // add ignore group (no collision between entities in same group)
// // rename to simulation
// figure out how to handle collisions
// shield

// TODO: Client:
// make sure body center of mass is at (0,0) with new shape translation
// // socket write do not need memcpy
// // add c++
// // impl binary decoder/encoder
// // add packet base class and one child for each packet type

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

    #[cfg(all(feature = "database", not(feature = "instance")))]
    {
        database::_start();
    }
    #[cfg(all(feature = "instance", not(feature = "database")))]
    {
        instance::_start();
    }
    #[cfg(all(feature = "database", feature = "instance"))]
    {
        std::thread::spawn(|| database::_start());
        instance::_start();
    }
}
