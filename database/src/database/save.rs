use super::*;
use bytes::Buf;
use serde::{Deserialize, Serialize};
use std::os::unix::fs::FileExt;

const SAVE_PATH: &str = "../../database.save";
const BACKUP_PATH: &str = "../../database.backup";

const PREFIX_CLIENT: u64 = 10;
const PREFIX_SIMULATION: u64 = 20;
const PREFIX_SHIP: u64 = 30;

const CAPACITIES_LEN: usize = 16;
const CAPACITIES: [u64; CAPACITIES_LEN] = [
    2u64.pow(6),
    2u64.pow(7),
    2u64.pow(8),
    2u64.pow(9),
    2u64.pow(10),
    2u64.pow(11),
    2u64.pow(12),
    2u64.pow(13),
    2u64.pow(14),
    2u64.pow(15),
    2u64.pow(16),
    2u64.pow(17),
    2u64.pow(18),
    2u64.pow(19),
    2u64.pow(20),
    2u64.pow(21),
];

pub struct Saver {
    sender: flume::Sender<(Key, Option<(u64, Vec<u8>)>)>,
    join_handle: std::thread::JoinHandle<()>,
}
impl Saver {
    pub fn start_and_populate(db: &mut Database) {
        assert!(db.saver.is_none());

        let mut clients = Vec::new();
        let mut simulations = Vec::new();
        let mut ships = Vec::new();

        match std::fs::read(SAVE_PATH) {
            Ok(buf) => {
                let mut buf = buf.as_slice();
                while buf.remaining() >= 8 * 4 {
                    let prev = buf;
                    let prefix = buf.get_u64_le();
                    let key = buf.get_u64_le();
                    let capacity = buf.get_u64_le() as usize;
                    let value_len = buf.get_u64_le() as usize;
                    let value = &buf[..value_len];
                    match prefix {
                        PREFIX_CLIENT => {
                            let id = ClientId::try_from_u64(key).unwrap();
                            let client = bin_decode::<ClientSave>(value).unwrap();
                            clients.push((id, client));
                            db.next_client_id = db
                                .next_client_id
                                .max(ClientId::try_from_u64(id.to_u64() + 1).unwrap());
                        }
                        PREFIX_SIMULATION => {
                            let id = SimulationId::try_from_u64(key).unwrap();
                            let simulation = bin_decode::<SimulationSave>(value).unwrap();
                            simulations.push((id, simulation));
                            db.next_simulation_id = db
                                .next_simulation_id
                                .max(SimulationId::try_from_u64(id.to_u64() + 1).unwrap());
                        }
                        PREFIX_SHIP => {
                            let id = ShipId::try_from_u64(key).unwrap();
                            let ship = bin_decode::<ShipSave>(value).unwrap();
                            ships.push((id, ship));
                            db.next_ship_id = db
                                .next_ship_id
                                .max(ShipId::try_from_u64(id.to_u64() + 1).unwrap());
                        }

                        _ => {}
                    };
                    buf = prev;
                    if buf.remaining() < capacity {
                        break;
                    }
                    buf.advance(capacity);
                }

                if let Err(err) = std::fs::rename(SAVE_PATH, BACKUP_PATH) {
                    log::warn!("Failed to rename save file: {}", err);
                }
            }
            Err(err) => log::warn!("Failed to read save file: {}", err),
        }

        let (sender, receiver) = flume::unbounded::<(Key, Option<(u64, Vec<u8>)>)>();
        let mut runner = SaverRunner {
            values: Default::default(),
            free: Default::default(),
            size_idx: CAPACITIES
                .iter()
                .enumerate()
                .map(|(i, &v)| (v, i))
                .collect(),
            file_len: 0,
            file: std::fs::File::create(SAVE_PATH).unwrap(),
        };
        let join_handle = std::thread::spawn(move || {
            while let Ok((key, value)) = receiver.recv() {
                if let Some((capacity, value)) = value {
                    // Insert
                    runner.insert(key, capacity, value);
                } else {
                    // Remove
                    runner.remove(key);
                }
            }
        });
        db.saver = Some(Self {
            sender,
            join_handle,
        });

        for (client_id, client) in clients {
            client.apply(client_id, db);
        }
        for (simulation_id, simulation) in simulations {
            simulation.apply(simulation_id, db);
        }
        for (ship_id, ship) in ships {
            ship.apply(ship_id, db);
        }
    }

    pub fn close(self) {
        std::mem::drop(self.sender);
        self.join_handle.join().unwrap();
    }

    fn encode(key_prefix: u64, key: u64, value: impl Serialize) -> (Key, Option<(u64, Vec<u8>)>) {
        let mut value = bin_encode(([0u8; 8 * 4], value));
        let capacity = (value.len() as u64).next_power_of_two().max(CAPACITIES[0]);
        let value_len = value.len() as u64 - 8 * 4;
        value[0..8].copy_from_slice(&key_prefix.to_le_bytes());
        value[8..16].copy_from_slice(&key.to_le_bytes());
        value[16..24].copy_from_slice(&capacity.to_le_bytes());
        value[24..32].copy_from_slice(&value_len.to_le_bytes());

        (Key { key_prefix, key }, Some((capacity, value)))
    }

    pub fn insert_client(&self, client_id: ClientId, client: &Client) {
        let _ = self.sender.send(Self::encode(
            PREFIX_CLIENT,
            client_id.to_u64(),
            ClientSave::save(client),
        ));
    }

    pub fn remove_client(&self, client_id: ClientId) {
        let _ = self.sender.send((
            Key {
                key_prefix: PREFIX_CLIENT,
                key: client_id.to_u64(),
            },
            None,
        ));
    }

    pub fn insert_simulation(&self, simulation_id: SimulationId, simulation: &Simulation) {
        let _ = self.sender.send(Self::encode(
            PREFIX_SIMULATION,
            simulation_id.to_u64(),
            SimulationSave::save(simulation),
        ));
    }

    pub fn remove_simulation(&self, simulation_id: SimulationId) {
        let _ = self.sender.send((
            Key {
                key_prefix: PREFIX_SIMULATION,
                key: simulation_id.to_u64(),
            },
            None,
        ));
    }

    pub fn insert_ship(&self, ship_id: ShipId, ship: &Ship) {
        let _ = self.sender.send(Self::encode(
            PREFIX_SHIP,
            ship_id.to_u64(),
            ShipSave::save(ship),
        ));
    }

    pub fn remove_ship(&self, ship_id: ShipId) {
        let _ = self.sender.send((
            Key {
                key_prefix: PREFIX_SHIP,
                key: ship_id.to_u64(),
            },
            None,
        ));
    }
}

#[derive(Default, Serialize, Deserialize)]
enum ClientSave {
    #[default]
    V0,
    V1 {
        password_sha256: [u8; 32],
    },
}
impl ClientSave {
    fn save(client: &Client) -> Self {
        Self::V1 {
            password_sha256: client.password_sha256,
        }
    }

    fn apply(self, client_id: ClientId, db: &mut Database) {
        match self {
            Self::V1 { password_sha256 } => {
                db.insert_client(client_id, password_sha256);
            }
            _ => {
                panic!("Unhandled ClientSave version");
            }
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
enum SimulationSave {
    #[default]
    V0,
}
impl SimulationSave {
    fn save(simulation: &Simulation) -> Self {
        Self::V0
    }

    fn apply(self, simulation_id: SimulationId, db: &mut Database) {
        match self {
            _ => {
                panic!("Unhandled SimulationSave version");
            }
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
enum ShipSave {
    #[default]
    V0,
    V1 {
        ship_data_id: ShipDataId,
        simulation_id: SimulationId,
        owner: Option<ClientId>,
        position: Vec2,
    },
}
impl ShipSave {
    fn save(ship: &Ship) -> Self {
        Self::V1 {
            ship_data_id: ship.ship_data_id,
            simulation_id: ship.simulation_id,
            owner: ship.owner,
            position: ship.position,
        }
    }

    fn apply(self, ship_id: ShipId, db: &mut Database) {
        match self {
            Self::V1 {
                ship_data_id,
                simulation_id,
                owner,
                position,
            } => {
                db.insert_ship(ship_id, ship_data_id, simulation_id, owner, position);
            }
            _ => {
                panic!("Unhandled ShipSave version");
            }
        }
    }
}

/// - key_prefix (0 = unused): u64
/// - key: u64
/// - capacity: u64
/// - value_len: u64
/// - value: [u8; value_len]
struct SaverRunner {
    values: HashMap<Key, FileValue>,
    free: [Vec<u64>; CAPACITIES_LEN],
    /// ex: 64 -> 0
    size_idx: HashMap<u64, usize>,
    file_len: u64,
    file: std::fs::File,
}
impl SaverRunner {
    fn insert(&mut self, key: Key, capacity: u64, value: Vec<u8>) {
        let position = match self.values.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if entry.get().capacity >= capacity {
                    entry.get().position
                } else {
                    if let Err(err) = self.file.write_all_at(&[0u8; 8], entry.get().position) {
                        log::error!("Failed to write: {}", err);
                        return;
                    }
                    self.free[self.size_idx[&capacity]].push(entry.get().position);

                    let position = self.free[self.size_idx[&capacity]]
                        .pop()
                        .unwrap_or_else(|| {
                            let position = self.file_len;
                            self.file_len += capacity;
                            position
                        });
                    *entry.get_mut() = FileValue { position, capacity };

                    position
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let position = self.free[self.size_idx[&capacity]]
                    .pop()
                    .unwrap_or_else(|| {
                        let position = self.file_len;
                        self.file_len += capacity;
                        position
                    });
                entry.insert(FileValue { position, capacity });

                position
            }
        };

        if let Err(err) = self.file.write_all_at(&value, position) {
            log::error!("Failed to write: {}", err);
        }
    }

    fn remove(&mut self, key: Key) {
        if let Some(value) = self.values.remove(&key) {
            if let Err(err) = self.file.write_all_at(&[0u8; 8], value.position) {
                log::error!("Failed to write: {}", err);
                return;
            }
            self.free[self.size_idx[&value.capacity]].push(value.position);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    key_prefix: u64,
    key: u64,
}

#[derive(Debug, Clone, Copy)]
struct FileValue {
    position: u64,
    capacity: u64,
}
