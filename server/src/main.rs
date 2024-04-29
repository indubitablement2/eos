mod asd;
mod interval;

use common::ids::*;
use flume::{unbounded, Receiver, Sender, TryRecvError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use common::connection::*;
use common::{HashMap, HashSet, IndexMap};

// type HashMap<K, V> = ahash::AHashMap<K, V>;
// type HashSet<K> = ahash::AHashSet<K>;
// type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

fn main() {
    common::logger::Logger::init();
    common::load_data();

    // TODO: Connect with database
    let database_connection = Connection::connect(common::DATABASE_ADDRESS).unwrap();

    // TODO: Database tells us what simulation to run

    let mut sim = common::simulation::Simulation::new(Default::default(), Default::default());
}
