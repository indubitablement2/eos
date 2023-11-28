pub mod entity;
pub mod entity_ai;
pub mod physics;

use super::*;
use entity::*;
use entity_ai::EntityAi;
use physics::*;
use rapier2d::prelude::*;

type SimRng = rand_xoshiro::Xoshiro128StarStar;
type Entities = IndexMap<EntityId, Entity, RandomState>;
type Ais = IndexMap<EntityId, EntityAi, RandomState>;

pub const DT: f32 = 1.0 / 20.0;
pub const DT_MS: u64 = 50;

// TODO: store collision events on entity for one tick

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub tick: u64,
    pub half_size: f32,
    rng: SimRng,

    pub physics: Physics,

    next_entity_id: EntityId,
    pub entities: Entities,
    pub ais: Ais,
}
impl Battlescape {
    pub fn new() -> Self {
        Self {
            rng: SimRng::from_entropy(),
            tick: 0,
            half_size: 100.0,
            physics: Default::default(),
            next_entity_id: EntityId(0),
            entities: Default::default(),
            ais: Default::default(),
        }
    }

    pub fn apply_cmd(&mut self, cmd: BattlescapeCommand) {
        // TODO
    }

    pub fn step(&mut self) {
        self.tick += 1;

        // Update ais.
        self.ais.retain(|entity_id, ai| {
            if let Some(entity_idx) = self.entities.get_index_of(entity_id) {
                !ai.update(entity_idx, &mut self.entities, &mut self.physics)
            } else {
                false
            }
        });

        // Update entities.
        self.entities.retain(|_entity_id, entity| {
            // TODO: Do something with the destroyed entity?
            !entity.update(&mut self.physics)
        });

        self.physics.step();

        // TODO: Handle physic events.
        let events = self.physics.take_events();
    }

    pub fn spawn_entity(
        &mut self,
        entity_data_id: EntityDataId,
        position: Isometry2<f32>,
    ) -> (EntityId, usize) {
        let entity_id = self.next_entity_id;
        self.next_entity_id.0 += 1;

        let entity = Entity::new(entity_data_id, entity_id, &mut self.physics, position);
        let entity_idx = self.entities.insert_full(entity_id, entity).0;

        if let Some(ai) = entity_data_id.data().ai {
            self.ais.insert_full(entity_id, ai);
            self.ais.entry(entity_id).or_insert(ai).changed(
                entity_idx,
                &mut self.entities,
                Default::default(),
            );
        }

        (entity_id, entity_idx)
    }

    fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(entity) = self.entities.swap_remove(&entity_id) {
            // TODO:
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum BattlescapeCommand {
    // TODO
}
