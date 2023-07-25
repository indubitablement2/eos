pub mod entity;
pub mod hull;
pub mod physics;

use super::*;
use physics::*;
use rapier2d::prelude::*;

use entity::*;
use hull::*;

pub type LocalEntityId = u16;
pub type HullIdx = u16;
pub type LocalTick = u32;

#[derive(Serialize, Deserialize, Default)]
pub struct System {
    #[serde(skip)]
    tick: LocalTick,

    physics: Physics,
    entities: AHashMap<LocalEntityId, Entity>,
    entity_id_dispenser: SmallIdDispenser,
}
impl System {
    pub fn new() -> Self {
        let mut s = Self::default();

        s
    }

    pub fn step(&mut self) {
        let mut removed_entities = Vec::new();
        for (id, entity) in self.entities.iter_mut() {
            entity.step();

            if entity.is_destroyed() {
                removed_entities.push(*id);
            }
        }
        for id in removed_entities {
            self.entities.remove(&id).unwrap().remove(&mut self.physics);
        }

        self.physics.step();
    }
}
