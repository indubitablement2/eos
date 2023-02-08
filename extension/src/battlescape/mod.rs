pub mod angle_vector;
pub mod bc_client;
pub mod bc_fleet;
pub mod command;
pub mod entity;
pub mod events;
pub mod mode;
pub mod physics;

use super::*;
use angle_vector::VectorAngle;
use godot::prelude::ToVariant;
use rand::prelude::*;
use rapier2d::prelude::*;

use bc_client::BattlescapeClient;
use bc_fleet::{BattlescapeFleet, FleetShipState};
use entity::ai::EntityAi;
use entity::{Entity, WishAngVel, WishLinVel};
use events::*;
use physics::*;

pub use self::mode::BattlescapeMode;
pub use command::*;

type SimRng = rand_xoshiro::Xoshiro128StarStar;
type Entities = IndexMap<EntityId, Entity, RandomState>;
type Clients = IndexMap<ClientId, BattlescapeClient, RandomState>;
pub type Fleets = IndexMap<FleetId, BattlescapeFleet, RandomState>;

pub const DT: f32 = 1.0 / 20.0;
pub const DT_MS: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(u32);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BattlescapeStateInit {
    pub seed: u64,
    pub mode: BattlescapeMode,
}

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub tick: u64,
    pub mode: BattlescapeMode,
    pub half_size: f32,
    /// Battle will end when timeout reach 0.
    pub end_timeout: u32,
    rng: SimRng,
    pub physics: Physics,

    #[serde(skip)]
    pub events: BattlescapeEventHandler,
    #[serde(skip)]
    new_entities: Vec<EntityId>,

    pub team_num_active_ship: AHashMap<u32, u32>,
    pub fleets: Fleets,
    pub clients: Clients,

    next_entity_id: EntityId,
    pub entities: Entities,
    pub ais: IndexMap<EntityId, EntityAi, RandomState>,
}
impl Battlescape {
    /// Amount of tick with active ships from only one team before battle is over.
    pub const END_TIMEOUT: u32 = (1.0 / DT) as u32 * 10;
    /// Amount of tick before end timeout may start.
    pub const END_TIMEOUT_START_TICK: u64 = (1.0 / DT) as u64 * 50;

    pub const SPAWN_OFFSET: f32 = 10.0;

    pub fn new(state_init: BattlescapeStateInit) -> Self {
        Self {
            rng: SimRng::seed_from_u64(state_init.seed),
            tick: Default::default(),
            mode: state_init.mode,
            half_size: 100.0,
            end_timeout: Self::END_TIMEOUT,
            physics: Default::default(),
            team_num_active_ship: Default::default(),
            fleets: Default::default(),
            clients: Default::default(),
            next_entity_id: EntityId(0),
            entities: Default::default(),
            ais: Default::default(),
            events: Default::default(),
            new_entities: Default::default(),
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
        let mut s = bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes).ok();

        if let Some(s) = &mut s {
            let bs_ptr = entity::script::BsPtr::new(s);

            for (entity, entity_idx) in s.entities.values_mut().zip(0usize..) {
                entity.post_deserialize_prepare(bs_ptr, entity_idx);
            }

            for entity in s.entities.values_mut() {
                entity.post_deserialize_post_prepare();
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
        let bs_ptr = entity::script::BsPtr::new(self);
        for (entity, entity_idx) in self.entities.values_mut().zip(0usize..) {
            entity.pre_step(bs_ptr, entity_idx);
        }

        // Call start on new entities.
        for entity_id in self.new_entities.drain(..) {
            if let Some(entity) = self.entities.get_mut(&entity_id) {
                entity.start();
            }
        }

        // Step entities.
        for entity in self.entities.values_mut() {
            entity.step(&mut self.physics);
        }

        let mut i = 0usize;
        while i < self.entities.len() {
            if self.entities[i].is_destroyed() {
                self.remove_entity(i);
            } else {
                i += 1;
            }
        }
    }

    fn add_fleet_ship(&mut self, fleet_id: FleetId, ship_idx: usize, prefered_spawn_point: usize) {
        if let Some(fleet) = self.fleets.get_mut(&fleet_id) {
            let entity_id = self.next_entity_id;

            let spawn_points = self.mode.spawn_points(fleet.team);
            let spawn_point = spawn_points
                .get(prefered_spawn_point)
                .unwrap_or_else(|| &spawn_points[0]);

            if let Some(entity) = fleet.try_spawn(
                ship_idx,
                spawn_point,
                self.half_size,
                entity_id,
                &mut self.physics,
            ) {
                self.next_entity_id.0 += 1;
                *self.team_num_active_ship.entry(fleet.team).or_default() += 1;
                let i = self.entities.insert_full(entity_id, entity).0;
                self.events.entity_added(entity_id, &self.entities[i]);
            }
        }
    }

    fn remove_entity(&mut self, entity_idx: usize) {
        if let Some((entity_id, entity)) = self.entities.swap_remove_index(entity_idx) {
            // Handle if this is a ship from a fleet.
            if let Some((fleet_id, ship_index)) = entity.fleet_ship {
                let fleet = self.fleets.get_mut(&fleet_id).unwrap();
                let fleet_ship = &mut fleet.ships[ship_index];

                if let Some(result) = entity.result() {
                    fleet_ship.state = FleetShipState::Removed(result);
                } else {
                    // Ship destroyed.
                    fleet_ship.state = FleetShipState::Destroyed;
                    self.events.ship_destroyed(fleet_id, ship_index);
                }

                *self.team_num_active_ship.get_mut(&entity.team).unwrap() -= 1;
            }

            self.events.entity_removed(entity_id, entity);
        }
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
