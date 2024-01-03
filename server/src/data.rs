use super::*;
use simulation::entity::{EntityData, EntityDataJson};
use std::{fs::File, io::BufReader};

const DATA_PATH: &str = "eos/client/tool/server_data.json";
const CONFIG_PATH: &str = "config.json";

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
    pub instances: AHashMap<InstanceId, InstanceData>,
    pub simulations: AHashMap<SimulationId, SimulationData>,
    pub entities: Vec<EntityData>,

    first_ship: usize,
}
impl Data {
    pub fn first_ship(&'static self) -> EntityDataId {
        EntityDataId(&self.entities[self.first_ship])
    }
}

pub struct InstanceData {
    pub addr: SocketAddr,
    pub simulations: Vec<SimulationId>,
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
    let mut instances = AHashMap::from_iter(json.instances.into_iter().map(
        |(instance_id, instance_addr)| {
            (
                instance_id,
                InstanceData {
                    addr: instance_addr.parse().unwrap(),
                    simulations: Vec::new(),
                },
            )
        },
    ));

    let simulations =
        AHashMap::from_iter(json.simulations.into_iter().map(|(id, simulation_json)| {
            instances
                .get_mut(&simulation_json.instance)
                .unwrap()
                .simulations
                .push(id);

            (
                id,
                SimulationData {
                    instance_id: simulation_json.instance,
                },
            )
        }));

    instances.retain(|instance_id, instance| {
        if instance.simulations.is_empty() {
            log::warn!("{:?} does not have any simulation", instance_id);
            false
        } else {
            true
        }
    });

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
        instances,
        simulations,
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
    instances: AHashMap<InstanceId, String>,
    simulations: AHashMap<SimulationId, SimulationDataJson>,
    entities: Vec<EntityDataJson>,
    first_ship: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct SimulationDataJson {
    instance: InstanceId,
}

// ####################################################################################
// ############## TEST ################################################################
// ####################################################################################

fn config_test() -> ConfigJson {
    ConfigJson {
        database_addr: "[::1]:0".to_string(),
        database_key: "key".to_string(),
    }
}

fn json_test() -> DataJson {
    let addresses = vec![
        "[2001::8a2e]:4993".to_string(),
        "[::1]:12345".to_string(),
        "[::]:3552".to_string(),
    ];

    let simulation_data_json = SimulationDataJson {
        instance: InstanceId::from_u32(1).unwrap(),
    };

    DataJson {
        instances: AHashMap::from_iter(
            addresses
                .into_iter()
                .enumerate()
                .map(|(i, addr)| (InstanceId::from_u32(i as u32 + 1).unwrap(), addr)),
        ),
        simulations: AHashMap::from_iter((1..4).map(|i| {
            (
                SimulationId::from_u32(i).unwrap(),
                simulation_data_json.clone(),
            )
        })),
        entities: vec![Default::default()],
        first_ship: 0,
    }
}

#[test]
fn test_asd() {
    println!("{}", serde_json::to_string_pretty(&json_test()).unwrap());
}
