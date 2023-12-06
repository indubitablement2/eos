pub mod entity;
pub mod physics;

use super::*;
use entity::*;
use physics::*;
use rapier2d::prelude::*;

type SimRng = rand_xoshiro::Xoshiro128StarStar;

pub const DT: f32 = 1.0 / 20.0;
pub const DT_MS: u64 = 50;

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub tick: u64,
    pub radius: f32,
    rng: SimRng,

    pub physics: Physics,

    next_entity_id: EntityId,
    pub entities: IndexMap<EntityId, Entity, RandomState>,

    /// Objects are processed in the same order they are added.
    objects: Vec<Object>,
}
impl Battlescape {
    pub fn new() -> Self {
        Self {
            rng: SimRng::from_entropy(),
            tick: 0,
            radius: 100.0,
            physics: Default::default(),
            next_entity_id: Default::default(),
            entities: Default::default(),
            objects: Default::default(),
        }
    }

    pub fn apply_cmd(&mut self, cmd: &BattlescapeCommand) {
        // TODO
    }

    pub fn step(&mut self) {
        self.tick += 1;

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
    }

    fn spawn_entity(
        &mut self,
        entity_data_id: EntityDataId,
        position: Isometry2<f32>,
        linvel: Vector2<f32>,
        angvel: f32,
        ignore: Option<EntityId>,
        target: Option<EntityId>,
    ) -> (EntityId, usize) {
        let entity_id = self.next_entity_id.next();

        let entity = Entity::new(
            self,
            entity_data_id,
            entity_id,
            position,
            linvel,
            angvel,
            ignore,
            target,
        );
        let entity_idx = self.entities.insert_full(entity_id, entity).0;

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
