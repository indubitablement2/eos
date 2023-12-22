use super::*;
use battlescape::entity::{EntityData, EntityDataJson};
use std::{fs::File, io::BufReader};

const DATA_PATH: &str = "eos/client/tool/data.json";
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
    pub systems: AHashMap<BattlescapeId, SystemData>,
    pub entities: Vec<EntityData>,
}

pub struct InstanceData {
    pub addr: SocketAddr,
    pub systems: Vec<BattlescapeId>,
}

pub struct SystemData {
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
                    systems: Vec::new(),
                },
            )
        },
    ));

    let systems = AHashMap::from_iter(json.systems.into_iter().map(|(id, system_json)| {
        instances
            .get_mut(&system_json.instance)
            .unwrap()
            .systems
            .push(id);

        (
            id,
            SystemData {
                instance_id: system_json.instance,
            },
        )
    }));

    instances.retain(|instance_id, instance| {
        if instance.systems.is_empty() {
            log::warn!("{:?} does not have any system", instance_id);
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

    Data {
        database_addr: config.database_addr.parse().unwrap(),
        database_key: config.database_key.into_bytes(),
        instances,
        systems,
        entities,
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
    systems: AHashMap<BattlescapeId, SystemDataJson>,
    entities: Vec<EntityDataJson>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SystemDataJson {
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

    let system_data_json = SystemDataJson {
        instance: InstanceId::from_u32(1).unwrap(),
    };

    DataJson {
        instances: AHashMap::from_iter(
            addresses
                .into_iter()
                .enumerate()
                .map(|(i, addr)| (InstanceId::from_u32(i as u32 + 1).unwrap(), addr)),
        ),
        systems: AHashMap::from_iter((1..4).map(|i| {
            (
                BattlescapeId::from_u64(i).unwrap(),
                system_data_json.clone(),
            )
        })),
        entities: vec![Default::default()],
    }
}

#[test]
fn test_asd() {
    println!("{}", serde_json::to_string_pretty(&json_test()).unwrap());
}
