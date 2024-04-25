pub mod connection;
pub mod database_packet;
pub mod ids;
pub mod logger;
pub mod simulation;

use anyhow::Context;
use connection::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smallvec::SmallVec;
use std::time::{Duration, Instant};

pub type HashMap<K, V> = ahash::AHashMap<K, V>;
pub type HashSet<K> = ahash::AHashSet<K>;
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

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
