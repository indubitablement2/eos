pub mod angle_vector;
pub mod bs_client;
pub mod bs_fleet;
pub mod command;
pub mod entity;
pub mod events;
pub mod physics;

use super::*;
use angle_vector::VectorAngle;
use rand::prelude::*;
use rapier2d::prelude::*;

use bs_client::BattlescapeClient;
use bs_fleet::{BattlescapeFleet, FleetShipState};
use entity::ai::EntityAi;
use entity::{Entity, EntityCondition, WishAngVel, WishLinVel};
use events::*;
use physics::*;

pub use command::*;

type SimRng = rand_xoshiro::Xoshiro128StarStar;
type Entities = IndexMap<EntityId, Entity, RandomState>;
type Clients = IndexMap<ClientId, BattlescapeClient, RandomState>;
pub type Fleets = IndexMap<FleetId, BattlescapeFleet, RandomState>;
pub type FleetShip = (FleetId, usize);
pub type Team = u32;

pub const DT: f32 = 1.0 / 20.0;
pub const DT_MS: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BattlescapeStateInit {
    pub seed: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub tick: u64,
    pub half_size: f32,
    /// Battle will end when timeout reach 0.
    pub end_timeout: u32,
    rng: SimRng,
    pub physics: Physics,

    #[serde(skip)]
    pub events: BattlescapeEventHandler,

    pub team_num_active_ship: AHashMap<Team, i32>,
    pub fleets: Fleets,
    pub clients: Clients,

    next_entity_id: EntityId,
    pub entities: Entities,
    pub ais: IndexMap<EntityId, EntityAi, RandomState>,
    // callbacks: Vec<>,
}
impl Battlescape {
    /// Amount of tick with active ships from only one team before battle is over.
    pub const END_TIMEOUT: u32 = (1.0 / DT) as u32 * 10;
    /// Amount of tick before end timeout may start.
    pub const END_TIMEOUT_START_TICK: u64 = (1.0 / DT) as u64 * 50;

    pub const SPAWN_OFFSET: f32 = 4.0;

    pub fn new(state_init: BattlescapeStateInit) -> Self {
        Self {
            rng: SimRng::seed_from_u64(state_init.seed),
            tick: Default::default(),
            half_size: 10.0,
            end_timeout: Self::END_TIMEOUT,
            physics: Default::default(),
            team_num_active_ship: Default::default(),
            fleets: Default::default(),
            clients: Default::default(),
            next_entity_id: EntityId(0),
            entities: Default::default(),
            ais: Default::default(),
            events: Default::default(),
        }
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        for entity in self.entities.values_mut() {
            entity.pre_serialize();
        }

        // Afaik this can not fail.
        bincode::Options::serialize(bincode::DefaultOptions::new(), self).unwrap()
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        let mut s: Option<Battlescape> =
            bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes).ok();

        if let Some(s) = &mut s {
            let bs_ptr = s.bs_ptr();

            for (entity, entity_idx) in s.entities.values_mut().zip(0usize..) {
                entity.post_deserialize(bs_ptr, entity_idx);
            }

            for entity in s.entities.values_mut() {
                entity.post_post_deserialize();
            }
        }

        s
    }

    /// Take the cmds for the tick `self.tick + 1`.
    pub fn step(
        &mut self,
        cmds: &Commands,
        mut events: BattlescapeEventHandler,
    ) -> BattlescapeEventHandler {
        if self.end_timout() {
            events.battle_over();
            return events;
        }

        self.events = events;

        self.tick += 1;

        cmds.apply(self);

        self.ai();
        self.step_entities();
        self.physics.step();
        // TODO: Handle physic events.

        if self.end_timeout == 0 {
            self.events.battle_over();
        }

        let mut events = std::mem::take(&mut self.events);
        events.stepped(&self);
        events
    }

    fn end_timout(&mut self) -> bool {
        if self.tick < Self::END_TIMEOUT_START_TICK {
            return false;
        }

        if self.end_timeout == 0 {
            return true;
        }

        if self
            .team_num_active_ship
            .values()
            .map(|num_active_ship| *num_active_ship > 0)
            .count()
            < 2
        {
            self.end_timeout -= 1;
        } else {
            self.end_timeout = Self::END_TIMEOUT;
        }

        false
    }

    fn ai(&mut self) {
        // Ais that want to be removed.
        let mut remove: Vec<EntityId> = Vec::new();

        for (entity_id, ai) in self.ais.iter_mut() {
            if ai.remove() {
                remove.push(*entity_id);
            } else if let Some((entity_index, _, _)) = self.entities.get_full(entity_id) {
                ai.update(
                    entity_index,
                    &mut self.entities,
                    &mut self.physics,
                    &mut self.clients,
                    &mut self.fleets,
                );
            } else {
                // No matching entity.
                remove.push(*entity_id);
            }
        }

        for entity_id in remove {
            self.ais.swap_remove(&entity_id);
        }
    }

    fn step_entities(&mut self) {
        // Prepare entities.
        let bs_ptr = self.bs_ptr();
        for (entity, entity_idx) in self.entities.values_mut().zip(0usize..) {
            entity.pre_step(bs_ptr, entity_idx);
        }

        // Step entities.
        let removed_entities = self
            .entities
            .values_mut()
            .enumerate()
            .filter_map(|(entity_idx, entity)| {
                if entity.step(&mut self.physics) {
                    Some(entity_idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Remove entities.
        for entity_idx in removed_entities.into_iter().rev() {
            self.remove_entity(entity_idx);
        }
    }

    fn add_fleet_ship(&mut self, fleet_id: FleetId, ship_idx: usize, prefered_spawn_point: usize) {
        let (condition, entity_data_id, owner, team) =
            if let Some(fleet) = self.fleets.get_mut(&fleet_id) {
                if let Some((condition, entity_data_id)) = fleet.try_spawn(ship_idx) {
                    self.events
                        .ship_state_changed(fleet_id, ship_idx, FleetShipState::Spawned);

                    (condition, entity_data_id, fleet.owner, fleet.team)
                } else {
                    return;
                }
            } else {
                return;
            };

        *self.team_num_active_ship.entry(team).or_default() += 1;

        let (translation, angle) = self.entity_spawn_point(team);
        self.add_entity(
            entity_data_id,
            translation,
            angle,
            Some((fleet_id, ship_idx)),
            owner,
            team,
            condition,
        );
    }

    fn add_entity(
        &mut self,
        entity_data_id: EntityDataId,
        translation: na::Vector2<f32>,
        angle: f32,
        fleet_ship: Option<FleetShip>,
        owner: Option<ClientId>,
        team: u32,
        condition: EntityCondition,
    ) -> EntityId {
        let entity_id = self.next_entity_id;
        self.next_entity_id.0 += 1;

        let entity_data = entity_data_id.data();

        let rb = self.physics.add_body(
            SimpleRigidBodyBuilder::dynamic()
                .translation(translation)
                .rotation(angle),
            BodyGenericId::EntityId(entity_id),
        );

        let hull_collider = self.physics.add_collider(
            SimpleColliderBuilder::new_ship(entity_data.shape.clone()).density(entity_data.density),
            rb,
            ColliderGenericId::Hull { entity_id },
        );

        let entity = Entity::new(
            entity_data_id,
            fleet_ship,
            owner,
            team,
            rb,
            hull_collider,
            condition,
        );

        let bs_ptr = self.bs_ptr();
        let entity_idx = self.entities.insert_full(entity_id, entity).0;
        let entity = &mut self.entities[entity_idx];
        entity.start(bs_ptr, entity_idx);

        self.events
            .entity_added(entity_id, &entity, translation, angle);

        entity_id
    }

    fn remove_entity(&mut self, entity_idx: usize) {
        if let Some((entity_id, entity)) = self.entities.swap_remove_index(entity_idx) {
            // Handle if this is a ship from a fleet.
            if let Some((fleet_id, ship_idx)) = entity.fleet_ship {
                let fleet_ship = &mut self.fleets.get_mut(&fleet_id).unwrap().ships[ship_idx];
                fleet_ship.condition = entity.condition();
                if entity.is_destroyed() {
                    fleet_ship.state = FleetShipState::Destroyed;
                } else {
                    fleet_ship.state = FleetShipState::Removed;
                }
                self.events
                    .ship_state_changed(fleet_id, ship_idx, fleet_ship.state);

                *self.team_num_active_ship.get_mut(&entity.team).unwrap() -= 1;
            }

            self.events.entity_removed(entity_id, entity);
        }
    }

    fn bs_ptr(&mut self) -> entity::script::BsPtr {
        entity::script::BsPtr::new(self)
    }

    pub fn spawn_point(&self, team: u32) -> (na::Vector2<f32>, f32) {
        match team {
            0 => (self.half_size * na::Vector2::new(0.0, -1.0), PI),
            1 => (self.half_size * na::Vector2::new(0.0, 1.0), 0.0),
            2 => (self.half_size * na::Vector2::new(-1.0, 0.0), FRAC_PI_2),
            _ => (self.half_size * na::Vector2::new(1.0, 0.0), -FRAC_PI_2),
        }
    }

    fn entity_spawn_point(&mut self, team: u32) -> (na::Vector2<f32>, f32) {
        // TODO: No overlapping.
        self.spawn_point(team)
    }

    pub fn entity_position(&self, entity_id: EntityId) -> Isometry<Real> {
        self.entities
            .get(&entity_id)
            .map(|entity| *self.physics.bodies[entity.rb].position())
            .unwrap_or_default()
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
