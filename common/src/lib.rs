pub mod connection;
pub mod database_packet;
pub mod ids;
pub mod logger;
pub mod simulation;

use ahash::{AHashMap, AHashSet, RandomState};
use anyhow::Context;
use connection::*;
use flume::{unbounded, Receiver, Sender, TryRecvError};
use ids::*;
use indexmap::IndexMap;
use rand::prelude::*;
use rapier2d::na::{self, Isometry2, Point2, UnitComplex, Vector2};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smallvec::SmallVec;
use std::net::SocketAddr;
use std::num::{NonZeroU32, NonZeroU64};
use std::time::{Duration, Instant};

static _TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
fn tokio() -> &'static tokio::runtime::Runtime {
    _TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

pub fn bin_encode(data: impl Serialize) -> Vec<u8> {
    postcard::to_allocvec(&data).unwrap()
}
pub fn bin_decode<T: DeserializeOwned>(data: &[u8]) -> anyhow::Result<T> {
    Ok(postcard::from_bytes(data)?)
}

pub fn load_data() {
    simulation::hull::data_json::load_hull_data();
}
