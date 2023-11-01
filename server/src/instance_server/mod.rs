use super::*;
use battlescape::*;
use bytes::{Buf, BufMut};

pub struct InstanceServer {
    central_server_connection: Connection,

    client_conection_receiver: std::sync::mpsc::Receiver<Connection>,

    client_login: Vec<(Connection, Option<ClientAuthPacket>)>,

    battlescapes: AHashMap<BattlescapeId, BattlescapeInner>,
}
impl InstanceServer {
    pub fn start() {
        Self {
            client_conection_receiver: Connection::bind_blocking(INSTANCE_ADDR_CLIENT),
            central_server_connection: Connection::connect_blocking(CENTRAL_ADDR_INSTANCE),
            battlescapes: Default::default(),
            client_login: Default::default(),
        }
        .run();
    }

    fn run(mut self) {
        log::info!("Instance server started");

        let mut now = std::time::Instant::now();
        let mut sim_time = 0.0f64;
        let mut real_time = 0.0f64;
        loop {
            real_time += now.elapsed().as_secs_f64();
            now = std::time::Instant::now();

            let dif = sim_time - real_time;
            if dif < -DT as f64 * 4.0 {
                log::warn!("Instance server is lagging behind by {} seconds", -dif);
                real_time = sim_time + DT as f64 * 4.0;
            } else if dif > 0.001 {
                std::thread::sleep(std::time::Duration::from_secs_f64(dif));
            }

            self.step();
            sim_time += DT as f64;

            if self.central_server_connection.disconnected {
                break;
            }
        }
    }

    fn step(&mut self) {
        let mut client_auth_response: AHashMap<ClientAuthPacket, bool> = Default::default();

        // Handle central server packets.
        while let Some(packet) = self
            .central_server_connection
            .recv::<CentralInstancePacket>()
        {
            match packet {
                CentralInstancePacket::CreateBattlescape { id } => {
                    self.battlescapes.insert(
                        id,
                        BattlescapeInner {
                            battlescape: Battlescape::new(),
                            client_connections: Default::default(),
                            step_since_no_connections: 0,
                        },
                    );
                }
                CentralInstancePacket::AuthClientResponse { auth, success } => {
                    client_auth_response.insert(auth, success);
                }
            }
        }

        // Take new client connection.
        for connection in self.client_conection_receiver.try_iter() {
            self.client_login.push((connection, None));
        }

        // Authenticate clients.
        let mut i = 0;
        while i < self.client_login.len() {
            let (connection, auth) = &mut self.client_login[i];
            if connection.disconnected {
                self.client_login.swap_remove(i);
                continue;
            }

            if let Some(auth) = *auth {
                if let Some(&result) = client_auth_response.get(&auth) {
                    let connection = self.client_login.swap_remove(i).0;

                    if result {
                        if let Some(battlescape) = self.battlescapes.get_mut(&auth.battlescape_id) {
                            battlescape
                                .client_connections
                                .push((connection, auth.client_id));
                            log::debug!("{:?} joined {:?}", auth.client_id, auth.battlescape_id);
                        }
                    }

                    continue;
                }
            } else {
                if let Some(client_login) = connection.recv::<ClientAuthPacket>() {
                    log::debug!("Received {:?}", client_login);
                    *auth = Some(client_login);
                    self.central_server_connection
                        .send(InstanceCentralPacket::AuthClient(client_login));
                }
            }

            i += 1;
        }

        // Handle client packets.
        for battlescape in self.battlescapes.values_mut() {
            for (connection, client_id) in battlescape.client_connections.iter_mut() {
                while let Some(packet) = connection.recv::<ClientInstancePacket>() {
                    match packet {
                        ClientInstancePacket::SpawnEntity {
                            entity_data_id,
                            translation,
                            angle,
                        } => {
                            battlescape
                                .battlescape
                                .spawn_entity(entity_data_id, Isometry2::new(translation, angle));
                        }
                    }
                }
            }
        }

        // TODO: Multithread this
        // Step battlescapes.
        let mut to_remove = Vec::new();
        for (&id, bc) in self.battlescapes.iter_mut() {
            bc.battlescape.step();

            if bc.client_connections.is_empty() {
                bc.step_since_no_connections += 1;
            } else {
                bc.step_since_no_connections = 0;
            }

            // TODO: Or if battlescape is over.
            if bc.step_since_no_connections > 800 {
                to_remove.push(id);
            }
        }
        for id in to_remove {
            self.battlescapes.remove(&id);
        }

        // Send state to clients.
        for battlescape in self.battlescapes.values_mut() {
            for (connection, _) in battlescape.client_connections.iter_mut() {
                let state = InstanceClientPacket::State {
                    tick: battlescape.battlescape.tick,
                    entity_positions: battlescape
                        .battlescape
                        .entities
                        .iter()
                        .map(|(&entity_id, entity)| {
                            let pos = battlescape.battlescape.physics.bodies[entity.rb].position();
                            (entity_id, pos.translation.vector, pos.rotation.angle())
                        })
                        .collect(),
                };

                connection.send(state);
            }
        }
    }
}

struct BattlescapeInner {
    battlescape: Battlescape,
    client_connections: Vec<(Connection, ClientId)>,
    step_since_no_connections: u32,
}

/// central -> instance
#[derive(Serialize, Deserialize)]
pub enum CentralInstancePacket {
    CreateBattlescape {
        id: BattlescapeId,
    },
    AuthClientResponse {
        auth: ClientAuthPacket,
        success: bool,
    },
}
impl SerializePacket for CentralInstancePacket {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}
impl DeserializePacket for CentralInstancePacket {
    fn deserialize(packet: &[u8]) -> Option<Self> {
        bincode::deserialize(packet).ok()
    }
}

/// instance -> central
#[derive(Serialize, Deserialize)]
pub enum InstanceCentralPacket {
    AuthClient(ClientAuthPacket),
}
impl SerializePacket for InstanceCentralPacket {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}
impl DeserializePacket for InstanceCentralPacket {
    fn deserialize(packet: &[u8]) -> Option<Self> {
        bincode::deserialize(packet).ok()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ClientAuthPacket {
    pub client_id: ClientId,
    pub token: u64,
    pub battlescape_id: BattlescapeId,
}
impl DeserializePacket for ClientAuthPacket {
    fn deserialize(mut packet: &[u8]) -> Option<Self> {
        if packet.remaining() != 24 {
            None
        } else {
            Some(Self {
                client_id: ClientId(packet.get_u64_le()),
                token: packet.get_u64_le(),
                battlescape_id: BattlescapeId(packet.get_u64_le()),
            })
        }
    }
}

/// instance -> client
enum InstanceClientPacket {
    State {
        tick: u64,
        entity_positions: Vec<(EntityId, Vector2<f32>, f32)>,
    },
}
impl SerializePacket for InstanceClientPacket {
    fn serialize(self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            InstanceClientPacket::State {
                tick,
                entity_positions,
            } => {
                buf.reserve_exact(4 + 8 + 4 + 12 * entity_positions.len());

                buf.put_u32_le(0);
                buf.put_u64_le(tick);
                buf.put_u32_le(entity_positions.len() as u32);

                for (entity_id, translation, rotation) in entity_positions {
                    buf.put_u64_le(entity_id.0);
                    buf.put_f32_le(translation.x);
                    buf.put_f32_le(translation.y);
                    buf.put_f32_le(rotation);
                }
            }
        }

        buf
    }
}

/// client -> instance
enum ClientInstancePacket {
    SpawnEntity {
        entity_data_id: EntityDataId,
        translation: Vector2<f32>,
        angle: f32,
    },
}
impl DeserializePacket for ClientInstancePacket {
    fn deserialize(mut packet: &[u8]) -> Option<Self> {
        if packet.remaining() < 4 {
            return None;
        }
        let packet_id = packet.get_u32_le();

        match packet_id {
            0 => {
                if packet.remaining() != 4 + 8 + 4 {
                    return None;
                }
                Some(Self::SpawnEntity {
                    entity_data_id: EntityDataId(packet.get_u32_le()),
                    translation: Vector2::new(packet.get_f32_le(), packet.get_f32_le()),
                    angle: packet.get_f32_le(),
                })
            }
            _ => None,
        }
    }
}
