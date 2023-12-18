use super::*;
use std::{fs::File, io::BufReader};

static DATA: std::sync::OnceLock<Data> = std::sync::OnceLock::new();
pub fn data() -> &'static Data {
    DATA.get().unwrap()
}

#[derive(Default)]
pub struct Data {
    pub instances: AHashMap<InstanceId, InstanceData>,
    pub systems: AHashMap<BattlescapeId, SystemData>,
}

pub struct InstanceData {
    pub addr: SocketAddr,
    pub systems: AHashSet<BattlescapeId>,
}

pub struct SystemData {
    pub instance_addr: InstanceId,
}

// ####################################################################################
// ############## JSON ################################################################
// ####################################################################################

pub fn _load_data() {
    let mut read = BufReader::new(File::open("data.json").unwrap());
    let json: DataJson = serde_json::from_reader(&mut read).unwrap();

    let mut data = Data::default();

    for (instance_id, instance_addr) in json.instances.into_iter() {
        data.instances.insert(
            instance_id,
            InstanceData {
                addr: instance_addr.parse::<SocketAddr>().unwrap(),
                systems: AHashSet::new(),
            },
        );
    }

    for (id, system_json) in json.systems.into_iter() {
        let system_data = SystemData {
            instance_addr: system_json.instance,
        };

        data.instances
            .get_mut(&system_json.instance)
            .unwrap()
            .systems
            .insert(id);

        data.systems.insert(id, system_data).unwrap();
    }

    data.instances.retain(|instance_id, instance| {
        if instance.systems.is_empty() {
            log::warn!("{:?} does not have any system", instance_id);
            false
        } else {
            true
        }
    });

    let _ = DATA.set(data);
}

pub fn _load_data_default() {
    let _ = DATA.set(Data::default());
}

#[derive(Serialize, Deserialize)]
struct DataJson {
    instances: AHashMap<InstanceId, String>,
    systems: AHashMap<BattlescapeId, SystemDataJson>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SystemDataJson {
    instance: InstanceId,
}

#[test]
fn test_asd() {
    let addresses = vec![
        "[2001::8a2e]:4993".to_string(),
        "[::1]:12345".to_string(),
        "[::]:0".to_string(),
    ];

    let system_data_json = SystemDataJson {
        instance: InstanceId::from_u32(1).unwrap(),
    };

    let json = DataJson {
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
    };

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
