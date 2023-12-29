use super::*;
use std::collections::VecDeque;

/// How long to keep an entity in known entities after it stop being seen.
const KNOWNS_ENTITY_TIMEOUT: f64 = 12.0;

// TODO: Add knows data
pub struct Client {
    connection: Connection,

    pub view_translation: Vector2<f32>,
    pub view_radius: f32,

    entity_id_allocator: NetworkIdAllocator,
    known_entities: AHashMap<EntityId, KnownEntity>,
}
impl Client {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            view_translation: Vector2::new(0.0, 0.0),
            view_radius: 20.0,
            entity_id_allocator: Default::default(),
            known_entities: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.entity_id_allocator = Default::default();
        self.known_entities.clear();
    }

    pub fn queue(&self, packet: ClientOutbound) {
        self.connection.queue(packet);
    }

    pub fn flush(&self) {
        self.connection.flush();
    }

    pub fn recv(&mut self) -> Result<ClientInbound, TryRecvError> {
        self.connection.recv()
    }

    pub fn close(&self, reason: &'static str) {
        self.connection.close(reason)
    }
}

struct KnownEntity {
    network_id: u32,
    distance_squared: f32,
    last_seen: f64,
    last_sent: f64,
    was_seen: bool,
}

/// Returns if the client should be retained.
pub fn update_client(sim: &mut Simulation, client_idx: usize) {
    let client = &mut sim.clients[client_idx];

    // Update what client can see.
    // TODO: detection
    sim.physics
        .query_pipeline
        .colliders_with_aabb_intersecting_aabb(
            &Aabb::from_half_extents(
                client.view_translation.into(),
                Vector2::new(client.view_radius, client.view_radius),
            ),
            |collider| {
                let collider = sim.physics.collider(*collider);

                let entity_id = collider.user_data.entity_id();

                let entity = client.known_entities.entry(entity_id).or_insert_with(|| {
                    let network_id = client.entity_id_allocator.next();

                    // Notify client of new entity.
                    client.connection.queue(ClientOutbound::AddEntity {
                        entity_id,
                        network_id,
                        entity_data_id: sim.entities[&entity_id].data,
                    });

                    KnownEntity {
                        network_id,
                        distance_squared: 0.0,
                        last_seen: 0.0,
                        last_sent: 0.0,
                        was_seen: true,
                    }
                });

                entity.distance_squared = (collider.position().translation.vector
                    - client.view_translation)
                    .magnitude_squared();
                entity.last_seen = sim.sim_time;

                true
            },
        );

    // Send state to clients.
    let time = sim.sim_time;
    let origin = client.view_translation;
    let mut entitie_states = Vec::new();
    client.known_entities.retain(|entity_id, known_entity| {
        if known_entity.last_seen == time {
            if !known_entity.was_seen {
                known_entity.was_seen = true;
                client.connection.queue(ClientOutbound::AddSeenEntity {
                    network_id: known_entity.network_id,
                });
            }

            // TODO: Send entities that are far away or not changing less often.

            let rb = sim.physics.body(sim.entities[entity_id].rb);
            let translation = *rb.translation();
            let rotation =
                rb.rotation()
                    .angle()
                    .mul_add(u16::MAX as f32 / TAU, u16::MAX as f32 * 0.5) as u16;

            entitie_states.push(EntityState {
                network_id: known_entity.network_id,
                translation,
                rotation,
            });

            known_entity.last_sent = time;

            true
        } else {
            if time - known_entity.last_seen > KNOWNS_ENTITY_TIMEOUT
                || !sim.entities.contains_key(entity_id)
            {
                client.connection.queue(ClientOutbound::RemoveEntity {
                    network_id: known_entity.network_id,
                });

                client.entity_id_allocator.free(known_entity.network_id);

                false
            } else {
                if known_entity.was_seen {
                    known_entity.was_seen = false;
                    client.connection.queue(ClientOutbound::RemoveSeenEntity {
                        network_id: known_entity.network_id,
                    });
                }

                true
            }
        }
    });

    client.queue(ClientOutbound::State {
        time,
        origin,
        entitie_states,
    });

    client.flush();
}

/// Dispense id < 128 first, then id >= 128.
#[derive(Default)]
struct NetworkIdAllocator {
    next_id: u32,
    free_ids: VecDeque<u32>,
}
impl NetworkIdAllocator {
    fn next(&mut self) -> u32 {
        self.free_ids.pop_front().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        })
    }

    fn free(&mut self, id: u32) {
        if id < 128 {
            self.free_ids.push_front(id)
        } else {
            self.free_ids.push_back(id)
        }
    }
}

#[derive(Serialize)]
pub struct EntityState {
    network_id: u32,
    // TODO: Pack tranlation into 2 i16
    translation: Vector2<f32>,
    rotation: u16,
}

#[derive(Serialize)]
pub enum ClientOutbound {
    EnteredSystem {
        client_id: ClientId,
        system_id: SimulationId,
    },
    State {
        time: f64,
        origin: Vector2<f32>,
        entitie_states: Vec<EntityState>,
    },
    AddEntity {
        entity_id: EntityId,
        network_id: u32,
        entity_data_id: EntityDataId,
    },
    RemoveEntity {
        network_id: u32,
    },
    RemoveSeenEntity {
        network_id: u32,
    },
    AddSeenEntity {
        network_id: u32,
    },
    ClientShips {
        ships: Vec<u8>,
    },
}
impl Packet for ClientOutbound {
    fn serialize(self) -> Vec<u8> {
        // TODO: custom serialize for step
        bin_encode(self)
    }

    fn parse(_buf: Vec<u8>) -> anyhow::Result<Self> {
        unimplemented!()
    }
}

#[derive(Deserialize)]
pub enum ClientInbound {
    Test,
}
impl Packet for ClientInbound {
    fn serialize(self) -> Vec<u8> {
        unimplemented!()
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bin_decode(&buf)
    }
}
