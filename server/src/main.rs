mod asd;
mod interval;

use common::ids::*;
use flume::{unbounded, Receiver, Sender, TryRecvError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

type HashMap<K, V> = ahash::AHashMap<K, V>;
type HashSet<K> = ahash::AHashSet<K>;
type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

#[tokio::main]
async fn main() {
    common::logger::Logger::init();

    // TODO: Connect with database
    // let database_connection = common::connection::Connection::connect(todo!(), "").unwrap();

    // TODO: Database tells us what simulation to run

    let mut sim = common::simulation::Simulation::new(Default::default(), Default::default());
}
