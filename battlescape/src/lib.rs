#![feature(slice_as_chunks)]
#![feature(duration_consts_float)]

pub mod commands;
mod fleet;
pub mod hull;
pub mod physics;
pub mod player_inputs;
mod schedule;
pub mod ship;
pub mod state_init;

extern crate nalgebra as na;

use ahash::{AHashMap, AHashSet};
use commands::BattlescapeCommand;
use indexmap::IndexMap;
use schedule::*;
use state_init::BattlescapeInitialState;
use std::time::Duration;

use fleet::*;
pub use rand::prelude::*;
pub use rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use smallvec::{smallvec, SmallVec};

pub use common::*;
pub use hull::*;
pub use physics::*;
pub use ship::*;

use crate::user_data::UserData;

type SimRng = rand_xoshiro::Xoshiro128StarStar;
type ShipSpawnQueue = AHashSet<(FleetId, usize)>;
pub type Team = u32;

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub bound: f32,
    pub tick: u64,
    rng: SimRng,
    pub physics: Physics,

    pub num_team: Team,

    pub fleets: IndexMap<FleetId, BattlescapeFleet>,
    pub next_ship_id: ShipId,

    pub ships: IndexMap<ShipId, BattlescapeShip, ahash::RandomState>,

    next_hull_id: HullId,
    pub hulls: IndexMap<HullId, Hull, ahash::RandomState>,
}
impl Battlescape {
    pub const TICK_DURATION: Duration = Duration::from_millis(50);
    pub const TICK_DURATION_SEC: f32 = Self::TICK_DURATION.as_secs_f32();
    pub const TICK_DURATION_MS: u32 = Self::TICK_DURATION.as_millis() as u32;

    pub fn new(battlescape_initial_state: BattlescapeInitialState) -> Self {
        Self {
            bound: battlescape_initial_state.bound,
            rng: SimRng::seed_from_u64(battlescape_initial_state.seed),
            tick: Default::default(),
            physics: Default::default(),
            num_team: Default::default(),
            fleets: Default::default(),
            next_ship_id: Default::default(),
            ships: Default::default(),
            next_hull_id: Default::default(),
            hulls: Default::default(),
        }
    }

    pub fn step(&mut self, cmds: &[BattlescapeCommand]) {
        let mut ship_spawn_queue = ShipSpawnQueue::new();

        apply_commands::apply_commands(self, cmds);
        debug_spawn_ships(self, &mut ship_spawn_queue);
        self.physics.step();
        // TODO: Handle events.

        self.process_ship_spawn_queue(ship_spawn_queue);

        self.tick += 1;
    }

    pub fn save(&self) -> Vec<u8> {
        // Afaik this can not fail.
        bincode::Options::serialize(bincode::DefaultOptions::new(), self).unwrap()
    }

    pub fn load(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes)
    }

    fn new_hull_id(&mut self) -> HullId {
        let id = self.next_hull_id;
        self.next_hull_id.0 += 1;
        id
    }

    fn new_ship_id(&mut self) -> ShipId {
        let id = self.next_ship_id;
        self.next_ship_id.0 += 1;
        id
    }

    pub fn ship_spawn_angle(&self, team: Team) -> f32 {
        (team as f32 / self.num_team as f32) * std::f32::consts::TAU
    }

    pub fn ship_spawn_rotation(&self, team: Team) -> f32 {
        self.ship_spawn_angle(team) + std::f32::consts::PI
    }

    pub fn ship_spawn_position(&self, team: Team) -> na::Vector2<f32> {
        na::UnitComplex::from_angle(self.ship_spawn_angle(team)) * na::vector![0.0, self.bound]
    }

    fn process_ship_spawn_queue(&mut self, ship_spawn_queue: ShipSpawnQueue) {
        for (fleet_id, index) in ship_spawn_queue {
            let (team, ship_data_id, ship_id) = if let Some(fleet) = self.fleets.get_mut(&fleet_id)
            {
                if let Some(ship_id) = fleet.available_ships.remove(&index) {
                    (
                        fleet.team,
                        fleet.original_fleet.ships[index].ship_data_id,
                        ship_id,
                    )
                } else {
                    log::warn!("Ship {:?}:{} is not available. Ignoring", fleet_id, index);
                    continue;
                }
            } else {
                log::warn!("{:?} does not exist. Ignoring", fleet_id);
                continue;
            };

            let ship_data = ship_data_id.data();
            let group_ignore = self.physics.new_group_ignore();
            let spawn_position = na::Isometry2::new(
                self.ship_spawn_position(team),
                self.ship_spawn_rotation(team),
            );

            let rb = RigidBodyBuilder::dynamic()
                .position(spawn_position)
                .user_data(UserData::build(
                    team,
                    group_ignore,
                    GenericId::ShipId(ship_id),
                    false,
                ))
                .build();
            let parrent_rb = self.physics.bodies.insert(rb);

            // Add hulls.
            let main_hull = self.add_hull(
                ship_data.main_hull,
                team,
                group_ignore,
                parrent_rb,
                GROUPS_SHIP,
            );
            let auxiliary_hulls: AuxiliaryHulls = ship_data
                .auxiliary_hulls
                .iter()
                .map(|&hull_data_id| {
                    self.add_hull(hull_data_id, team, group_ignore, parrent_rb, GROUPS_SHIP)
                })
                .collect();

            self.ships.insert(
                ship_id,
                BattlescapeShip {
                    fleet_id,
                    index,
                    ship_data_id,
                    rb: parrent_rb,
                    mobility: ship_data.mobility,
                    main_hull,
                    auxiliary_hulls,
                },
            );
        }
    }

    fn add_hull(
        &mut self,
        hull_data_id: HullDataId,
        team: u32,
        group_ignore: u32,
        parrent_rb: RigidBodyHandle,
        groups: InteractionGroups,
    ) -> HullId {
        let hull_data = hull_data_id.data();
        let hull_id = self.new_hull_id();
        let user_data =
            UserData::build(team, group_ignore, GenericId::from_hull_id(hull_id), false);
        let coll = build_hull_collider(hull_data_id, groups, user_data);
        let coll_handle = self.physics.insert_collider(parrent_rb, coll);
        self.hulls.insert(
            hull_id,
            Hull {
                hull_data_id,
                current_defence: hull_data.defence,
                collider: coll_handle,
            },
        );
        hull_id
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[deprecated]
fn debug_spawn_ships(bc: &mut Battlescape, ship_queue: &mut ShipSpawnQueue) {
    for (fleet_id, fleet) in bc.fleets.iter() {
        for ship_index in fleet.available_ships.keys() {
            ship_queue.insert((*fleet_id, *ship_index));
            break;
        }
    }
}
