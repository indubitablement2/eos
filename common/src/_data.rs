use super::*;
use simulation::entity::{EntityData, EntityDataJson};
use std::{fs::File, io::BufReader};

const DATA_PATH: &str = "../client/tool/server_data.json";
const CONFIG_PATH: &str = "../../config.json";

static DATA: std::sync::OnceLock<Data> = std::sync::OnceLock::new();
pub fn data() -> &'static Data {
    DATA.get_or_init(|| {
        log::error!("Data set for test");
        parse_json(config_test(), json_test())
    })
}

pub struct Data {
    pub database_addr: SocketAddr,
    pub database_key: Vec<u8>,
    pub entities: Vec<EntityData>,

    first_ship: usize,
}
impl Data {
    pub fn first_ship(&'static self) -> EntityDataId {
        EntityDataId(&self.entities[self.first_ship])
    }
}

pub struct SimulationData {
    pub instance_id: InstanceId,
}

// ####################################################################################
// ############## LOAD ################################################################
// ####################################################################################

pub fn load_data() {
    let mut read = BufReader::new(File::open(DATA_PATH).unwrap());
    let data = parse_json(
        serde_json::from_slice(&std::fs::read(CONFIG_PATH).unwrap()).unwrap(),
        serde_json::from_reader(&mut read).unwrap(),
    );

    if DATA.set(data).is_err() {
        log::error!("Data already set");
    } else {
        log::info!("Data loaded properly");
    }
}

fn parse_json(config: ConfigJson, json: DataJson) -> Data {
    let entities = json
        .entities
        .into_iter()
        .zip(0u32..)
        .map(|(entity_json, id)| entity_json.parse(id))
        .collect::<Vec<_>>();

    let first_ship = json.first_ship;
    assert!(first_ship < entities.len());

    Data {
        database_addr: config.database_addr.parse().unwrap(),
        database_key: config.database_key.into_bytes(),
        entities,
        first_ship,
    }
}

// ####################################################################################
// ############## JSON ################################################################
// ####################################################################################

/// Kept secrets.
#[derive(Serialize, Deserialize)]
struct ConfigJson {
    database_addr: String,
    database_key: String,
}

#[derive(Serialize, Deserialize)]
struct DataJson {
    entities: Vec<EntityDataJson>,
    first_ship: usize,
}

// ####################################################################################
// ############## TEST ################################################################
// ####################################################################################

fn config_test() -> ConfigJson {
    ConfigJson {
        database_addr: "[::1]:14729".to_string(),
        database_key: "key".to_string(),
    }
}

fn json_test() -> DataJson {
    DataJson {
        entities: vec![Default::default()],
        first_ship: 0,
    }
}

#[test]
fn test_data_json() {
    println!("{}", serde_json::to_string_pretty(&json_test()).unwrap());
}
