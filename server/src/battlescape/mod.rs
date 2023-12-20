pub mod client;
pub mod entity;
pub mod physics;

use super::*;
use client::*;
use entity::*;
use physics::*;
use rapier2d::prelude::*;

type SimRng = rand_xoshiro::Xoshiro128StarStar;

pub const DT: f32 = 1.0 / 20.0;
pub const DT_MS: u64 = 50;

/// How many tick between battlescape saves. (30 minutes)
const SAVE_INTERVAL: u64 = 30 * 60 * (1000 / DT_MS);

const RADIUS: f32 = 100.0;

pub enum BattlescapeInbound {
    DatabaseBattlescapeResponse(DatabaseBattlescapeResponse),
    NewClient { client_id: ClientId, client: Client },
}

pub struct Battlescape {
    pub battlescape_id: BattlescapeId,

    /// Seconds since unix epoch of last step.
    global_time: f64,
    pub tick: u64,
    next_save_tick: u64,
    rng: SimRng,

    pub physics: Physics,

    next_entity_id: EntityId,
    pub entities: IndexMap<EntityId, Entity, RandomState>,

    /// Objects are processed in the same order they are added.
    objects: Vec<Object>,

    database_outbound: ConnectionOutbound,
    battlescape_inbound: Receiver<BattlescapeInbound>,

    clients: IndexMap<ClientId, Client, RandomState>,
}
impl Battlescape {
    pub fn new(
        battlescape_id: BattlescapeId,
        database_outbound: ConnectionOutbound,
        battlescape_inbound: Receiver<BattlescapeInbound>,
        save: BattlescapeMiscSave,
    ) -> Self {
        Self {
            tick: 0,
            rng: SimRng::from_entropy(),
            physics: Default::default(),
            next_entity_id: Default::default(),
            entities: Default::default(),
            objects: Default::default(),
            clients: Default::default(),
            battlescape_id,

            global_time: global_time(),
            next_save_tick: thread_rng().gen_range(2000..8000),
            database_outbound,
            battlescape_inbound,
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.global_time = global_time();

        // TODO: Handle inbound.

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

        // Save.
        if self.tick > self.next_save_tick {
            self.save();
        }
    }

    fn save(&mut self) {
        self.next_save_tick = self.tick + SAVE_INTERVAL + self.rng.gen_range(0..4000);

        let misc = BattlescapeMiscSave {};

        self.database_outbound
            .queue(DatabaseRequest::SaveBattlescape {
                battlescape_id: self.battlescape_id,
                battlescape_misc_save: bin_encode(&misc),
            });
        // TODO: Save ships
        // TODO: Save planets?
    }

    fn spawn_entity(
        &mut self,
        data: &'static EntityData,
        save: EntitySave,
        ignore: Option<EntityId>,
        target: Option<EntityId>,
    ) -> (EntityId, usize) {
        let entity_id = self.next_entity_id.next();

        let entity = Entity::new(self, data, save, entity_id, ignore, target);
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
    fn update_retain(&mut self, battlescape: &mut Battlescape) -> bool {
        match self {
            Self::Seek { entity_id } => {
                let Some((entity_idx, _, entity)) = battlescape.entities.get_full(entity_id) else {
                    return false;
                };

                // Map to target's translation.
                if let Some(target) = entity
                    .target
                    .and_then(|target| battlescape.entities.get(&target))
                    .map(|target| *battlescape.physics.body(target.rb).translation())
                {
                    battlescape.entities[entity_idx].wish_angvel = WishAngVel::AimSmooth(target);
                }

                true
            }
            Self::Ship { entity_id } => {
                let Some((entity_idx, _, entity)) = battlescape.entities.get_full(entity_id) else {
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

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct BattlescapeMiscSave {
    // TODO: Debris
    // TODO: items
}
impl Default for BattlescapeMiscSave {
    fn default() -> Self {
        Self {}
    }
}
