pub mod connection;
pub mod database_packet;
pub mod ids;
pub mod interval;
pub mod logger;
pub mod simulation;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use smallvec::SmallVec;
use std::time::Duration;

pub type HashMap<K, V> = ahash::AHashMap<K, V>;
pub type HashSet<K> = ahash::AHashSet<K>;
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

pub const DATABASE_ADDRESS: &str = "ws://127.0.0.1:43598";

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

pub fn bit_encode(data: &impl bitcode::Encode) -> Vec<u8> {
    bitcode::encode(data)
}
pub fn bit_decode<T: bitcode::DecodeOwned>(buf: &[u8]) -> Result<T, bitcode::Error> {
    bitcode::decode(buf)
}

pub fn load_data() {
    simulation::load_data();
}
