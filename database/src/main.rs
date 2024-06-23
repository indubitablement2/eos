mod client;
mod database;
mod ids;
mod listener;

use database::Database;
use ids::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

type HashMap<K, V> = ahash::AHashMap<K, V>;
type HashSet<K> = ahash::AHashSet<K>;
type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;
type DashMap<K, V> = dashmap::DashMap<K, V, ahash::RandomState>;

fn bin_encode(data: impl Serialize) -> Vec<u8> {
    postcard::to_allocvec(&data).unwrap()
}
fn bin_encode_into(data: impl Serialize, writer: impl std::io::Write) {
    postcard::to_io(&data, writer).unwrap();
}
fn bin_decode<T: DeserializeOwned>(data: &[u8]) -> anyhow::Result<T> {
    Ok(postcard::from_bytes(data)?)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    log::info!("Server starting");
    Database::_load();

    log::info!("Server started");
    listener::listener_loop("addr").await;

    // database.save();
}
