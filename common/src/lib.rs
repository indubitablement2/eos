pub mod connection;
pub mod database_packet;
pub mod ids;
pub mod interval;
pub mod logger;
pub mod ship;
pub mod simulation;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smallvec::SmallVec;
use std::time::Duration;

pub type Vec2 = glam::Vec2;
pub type IVec2 = glam::i32::IVec2;
pub type HashMap<K, V> = ahash::AHashMap<K, V>;
pub type HashSet<K> = ahash::AHashSet<K>;
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

static _TOKIO_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
pub fn tokio() -> &'static tokio::runtime::Runtime {
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
pub fn bin_encode_into(data: impl Serialize, writer: impl std::io::Write) {
    postcard::to_io(&data, writer).unwrap();
}
pub fn bin_decode<T: DeserializeOwned>(data: &[u8]) -> anyhow::Result<T> {
    Ok(postcard::from_bytes(data)?)
}

pub fn database_address() -> String {
    std::env::var("DATABASE_ADDRESS").unwrap()
}

pub fn load_data() {
    simulation::load_data();
    ship::load_data();
}
