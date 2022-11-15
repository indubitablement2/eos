#![feature(variant_count)]

pub mod command;
pub mod hull;
pub mod physics;
pub mod render_event;
pub mod ship;
pub mod sim_events;
mod schedule;

use ahash::{AHashMap, AHashSet};
use indexmap::IndexMap;
use num_enum::{FromPrimitive, IntoPrimitive};
use rand::prelude::*;
use rapier2d::na::{self, ComplexField, RealField};
use rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use hull::data::HullDataId;
use physics::*;
use render_event::RenderEvents;
use ship::Ship;
use sim_events::SimulationEvents;

pub use command::*;
pub use ship::{data::ShipDataId, ShipId};

/// Timestep length in seconds.
pub const DT: f32 = 0.1;

type SimRng = rand_xoshiro::Xoshiro128StarStar;
pub type SystemId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

#[derive(Serialize, Deserialize)]
pub struct Simulation {
    pub system_id: SystemId,
    pub bound: f32,
    pub tick: u64,
    rng: SimRng,
    pub physics: Physics,

    // pub clients: IndexMap<ClientId, BattlescapeClient, ahash::RandomState>,
    // pub fleets: IndexMap<FleetId, BattlescapeFleet, ahash::RandomState>,
    pub next_ship_id: ShipId,
    pub ships: IndexMap<ShipId, Ship, ahash::RandomState>,
}
impl Simulation {
    pub fn new(system_id: SystemId, bound: f32) -> Self {
        Self {
            system_id,
            bound,
            rng: SimRng::from_entropy(),
            tick: Default::default(),
            physics: Default::default(),
            next_ship_id: ShipId((system_id as u64) << 48),
            ships: Default::default(),
        }
    }

    pub fn step(&mut self, cmds: &[Command]) -> (RenderEvents, SimulationEvents) {
        self._step(cmds)
    }

    pub fn serialize(&self) -> Vec<u8> {
        // Afaik this can not fail.
        bincode::Options::serialize(bincode::DefaultOptions::new(), self).unwrap()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes)
    }

    fn new_ship_id(&mut self) -> ShipId {
        let id = self.next_ship_id;
        self.next_ship_id.0 += 1;
        id
    }

    // pub fn ship_spawn_angle(&self, team: Team) -> f32 {
    //     (team as f32 / self.num_team as f32) * std::f32::consts::TAU
    // }

    // pub fn ship_spawn_rotation(&self, team: Team) -> f32 {
    //     self.ship_spawn_angle(team) + std::f32::consts::PI
    // }

    // pub fn ship_spawn_position(&self, team: Team) -> na::Vector2<f32> {
    //     na::UnitComplex::from_angle(self.ship_spawn_angle(team)) * na::vector![0.0, self.bound]
    // }
}
