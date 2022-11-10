#![feature(slice_as_chunks)]
#![feature(duration_consts_float)]

pub mod commands;
pub mod fleet;
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
use state_init::BattlescapeInitialState;
use std::time::Duration;
use rand::prelude::*;
use rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use common::*;
use user_data::*;

pub use fleet::*;
pub use ship::*;
pub use hull::*;
pub use physics::*;

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
        self._step(cmds);
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

    fn new_team(&mut self) -> Team {
        let team = self.num_team + 1;
        self.num_team += 1;
        team
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
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
