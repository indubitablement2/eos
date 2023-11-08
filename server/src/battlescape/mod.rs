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

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub battlescape_id: BattlescapeId,

    /// Moving average duration of a step.
    pub step_duration: f32,

    pub tick: u64,
    pub half_size: f32,
    rng: SimRng,
    pub physics: Physics,

    next_entity_id: EntityId,

    pub entities: Entities,
    pub ais: IndexMap<EntityId, EntityAi, RandomState>,
}
impl Battlescape {
    pub fn new(battlescape_id: BattlescapeId) -> Self {
        Self {
            battlescape_id,
            step_duration: 0.005,
            rng: SimRng::from_entropy(),
            tick: 0,
            half_size: 100.0,
            physics: Default::default(),
            next_entity_id: EntityId(0),
            entities: Default::default(),
            ais: Default::default(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        // Afaik this can not fail.
        serde_json::to_vec(&self).unwrap()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    pub fn step(&mut self, now: &mut Instant, delta: f32) {
        self.tick += 1;

        // Update ais.
        let mut remove: Vec<usize> = Vec::new();
        for (ai_idx, (entity_id, ai)) in self.ais.iter_mut().enumerate() {
            if let Some(entity_idx) = self.entities.get_index_of(entity_id) {
                if ai.update(entity_idx, &mut self.entities, &mut self.physics) {
                    remove.push(ai_idx);
                }
            }
        }
        for ai_idx in remove.into_iter().rev() {
            self.ais.swap_remove_index(ai_idx);
        }

        // Update entities.
        let mut remove: Vec<usize> = Vec::new();
        for (entity_idx, entity) in self.entities.values_mut().enumerate() {
            if entity.update(&mut self.physics) {
                remove.push(entity_idx);
            }
        }
        for entity_idx in remove.into_iter().rev() {
            if let Some((entity_id, entity)) = self.entities.swap_remove_index(entity_idx) {
                // TODO: Do something with the destroyed entity?
                self.ais.swap_remove(&entity_id);
            };
        }

        self.physics.step();
        // TODO: Handle physic events.

        let new_now = Instant::now();
        let elapsed = (new_now - *now).as_secs_f32();
        self.step_duration = self.step_duration * 0.98 + elapsed * 0.02;
        *now = new_now;
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
