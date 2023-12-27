pub mod client;
pub mod entity;
pub mod physics;

use super::*;
use client::*;
use entity::*;
use physics::*;
use rapier2d::prelude::*;
use std::ops::Range;

type SimRng = rand_xoshiro::Xoshiro128StarStar;

pub const DT: f32 = 1.0 / 20.0;
pub const DT_MS: u64 = 50;
const TICK_PER_SECOND: u64 = 1000 / DT_MS;

/// How many tick between simulation saves. (30 minutes)
const SAVE_INTERVAL: u64 = 30 * 60 * TICK_PER_SECOND;
/// Add some randomness to stagger saves.
const SAVE_INTERVAL_RANDOMNESS: Range<u64> = 0..4096;

const RADIUS: f32 = 100.0;

pub enum SimulationInbound {
    DatabaseSimulationResponse(DatabaseSimulationResponse),
    NewClient { client_id: ClientId, client: Client },
    SaveRequest,
}

pub struct Simulation {
    simulation_id: SimulationId,

    /// Seconds since unix epoch of current step.
    global_time: f64,
    tick: u64,
    next_save_tick: u64,
    rng: SimRng,

    physics: Physics,

    next_entity_id: EntityId,
    entities: IndexMap<EntityId, Entity, RandomState>,

    /// Objects are processed in the same order they are added.
    objects: Vec<Object>,

    database_outbound: ConnectionOutbound,
    simulation_inbound: Receiver<SimulationInbound>,

    clients: IndexMap<ClientId, Client, RandomState>,
}
impl Simulation {
    pub fn new(
        simulation_id: SimulationId,
        database_outbound: ConnectionOutbound,
        simulation_inbound: Receiver<SimulationInbound>,
        save: SimulationSave,
    ) -> Self {
        Self {
            tick: 0,
            rng: SimRng::from_entropy(),
            physics: Default::default(),
            next_entity_id: Default::default(),
            entities: Default::default(),
            objects: Default::default(),
            clients: Default::default(),
            simulation_id,

            global_time: global_time(),
            next_save_tick: thread_rng().gen_range(SAVE_INTERVAL_RANDOMNESS),
            database_outbound,
            simulation_inbound,
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.global_time = global_time();

        // Handle inbound.
        while let Ok(inbound) = self.simulation_inbound.try_recv() {
            match inbound {
                SimulationInbound::DatabaseSimulationResponse(response) => match response {
                    DatabaseSimulationResponse::ClientShips {
                        client_id,
                        client_ships,
                    } => {
                        if let Some(client) = self.clients.get(&client_id) {
                            client.connection.queue(ClientOutbound::ClientShips {
                                ships: client_ships,
                            });
                        }
                    }
                    DatabaseSimulationResponse::ShipEntered {
                        ship_id,
                        save: entity_save,
                    } => {
                        self.spawn_entity(entity_save, None, None, Some(ship_id));
                    }
                },
                SimulationInbound::NewClient { client_id, client } => {
                    client.connection.queue(ClientOutbound::EnteredSystem {
                        client_id,
                        system_id: self.simulation_id,
                    });
                    self.clients.insert(client_id, client);
                }
                SimulationInbound::SaveRequest => {
                    self.save();
                }
            }
        }

        // Handle client packets.
        self.clients.retain(|client_id, client| loop {
            match client.connection.recv::<ClientInbound>() {
                Ok(packet) => match packet {
                    ClientInbound::Test => {}
                },
                Err(TryRecvError::Empty) => break true,
                Err(TryRecvError::Disconnected) => break false,
            }
        });

        self.physics.step();

        // TODO Handle physic events.
        for (a, event) in self.physics.events.0.try_lock().unwrap().iter().copied() {
            // if let Some(entity) = self.entities.get_mut(&a) {
            //     entity.take_contact_event(event);
            // }

            // let b = event.with_entity_id;
            // let event = ContactEvent {
            //     collider_id: event.with_collider_id,
            //     with_entity_id: a,
            //     with_collider_id: event.collider_id,
            //     force_direction: event.force_direction,
            //     force_magnitude: event.force_magnitude,
            // };
            // if let Some(entity) = self.entities.get_mut(&b) {
            //     entity.take_contact_event(event);
            // }
        }

        // Update entities.
        self.entities.retain(|_entity_id, entity| {
            // TODO: Do something with the destroyed entity?
            !entity.update(&mut self.physics)
        });

        // Update objects.
        let mut objs = std::mem::take(&mut self.objects);
        objs.retain_mut(|obj| obj.update_retain(self));
        std::mem::swap(&mut self.objects, &mut objs);
        // Add new objects.
        self.objects.extend(objs.into_iter());

        // TODO: Send state to clients.
        for client in self.clients.values_mut() {
            client.connection.flush();
        }

        // Save.
        if self.tick > self.next_save_tick {
            self.save();
        }
    }

    fn save(&mut self) {
        self.next_save_tick =
            self.tick + SAVE_INTERVAL + self.rng.gen_range(SAVE_INTERVAL_RANDOMNESS);

        let simulation_save = SimulationSave {};

        self.database_outbound
            .queue(DatabaseRequest::SaveSimulation {
                simulation_id: self.simulation_id,
                simulation_save,
            });
        // TODO: Save ships
        // TODO: Save planets?
    }

    fn spawn_entity(
        &mut self,
        save: EntitySave,
        ignore: Option<EntityId>,
        target: Option<EntityId>,
        ship_id: Option<ShipId>,
    ) -> (EntityId, usize) {
        let entity_id = if let Some(ship_id) = ship_id {
            ship_id.to_entity_id()
        } else {
            self.next_entity_id.next()
        };

        let entity = Entity::new(self, save, entity_id, ignore, target);
        let entity_idx = self.entities.insert_full(entity_id, entity).0;

        (entity_id, entity_idx)
    }

    fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(entity) = self.entities.swap_remove(&entity_id) {
            // TODO:
        }
    }
}

fn global_time() -> f64 {
    std::time::UNIX_EPOCH
        .elapsed()
        .unwrap_or_default()
        .as_secs_f64()
}

/// Something that modify the simulation (ai, effect, etc).
#[derive(Debug, Serialize, Deserialize)]
enum Object {
    /// Will try to face entity's target and go forward at max speed.
    /// If entity has no target just move forward untill a target is set.
    Seek {
        entity_id: EntityId,
    },
    Ship {
        entity_id: EntityId,
    },
}
impl Object {
    fn new_seek(entity: &mut Entity, entity_id: EntityId) -> Self {
        entity.wish_linvel = WishLinVel::ForceRelative(Vector2::new(1.0, 0.0));

        Self::Seek { entity_id }
    }

    // Removed if returning `false`.
    fn update_retain(&mut self, simulation: &mut Simulation) -> bool {
        match self {
            Self::Seek { entity_id } => {
                let Some((entity_idx, _, entity)) = simulation.entities.get_full(entity_id) else {
                    return false;
                };

                // Map to target's translation.
                if let Some(target) = entity
                    .target
                    .and_then(|target| simulation.entities.get(&target))
                    .map(|target| *simulation.physics.body(target.rb).translation())
                {
                    simulation.entities[entity_idx].wish_angvel = WishAngVel::AimSmooth(target);
                }

                true
            }
            Self::Ship { entity_id } => {
                let Some((entity_idx, _, entity)) = simulation.entities.get_full(entity_id) else {
                    return false;
                };

                if entity.controlled {
                    // TODO
                } else {
                    // TODO
                }

                true
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SimulationSave {
    // TODO: Debris
    // TODO: items
    // TODO: planets state
}
impl Default for SimulationSave {
    fn default() -> Self {
        Self {}
    }
}
