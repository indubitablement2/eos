#![feature(slice_as_chunks)]
#![feature(duration_consts_float)]

pub mod commands;
pub mod hull;
pub mod physics;
pub mod player_inputs;
mod schedule;
pub mod ship;
pub mod state_init;

extern crate nalgebra as na;

use commands::BattlescapeCommand;
use indexmap::IndexMap;
use schedule::*;
use state_init::BattlescapeInitialState;
use std::time::Duration;

pub use rand::prelude::*;
pub use rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use smallvec::{smallvec, SmallVec};

pub use data::*;
pub use hull::*;
pub use physics::*;
pub use ship::*;

type SimRng = rand_xoshiro::Xoshiro128StarStar;

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub bound: f32,
    pub tick: u64,
    rng: SimRng,
    pub physics: Physics,

    next_ship_id: ShipId,
    pub ships: IndexMap<ShipId, Ship, ahash::RandomState>,

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
            tick: 0,
            physics: Default::default(),
            next_ship_id: Default::default(),
            ships: Default::default(),
            next_hull_id: Default::default(),
            hulls: Default::default(),
        }
    }

    pub fn step(&mut self, cmds: &[BattlescapeCommand]) {
        let mut ship_queue = ShipSpawnQueue::new(self);

        apply_commands::apply_commands(self, cmds);
        debug_spawn_ships(self, &mut ship_queue);
        self.physics.step();
        // TODO: Handle events.

        ship_queue.process(self);

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
        self.next_ship_id.0 += 1;
        id
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[deprecated]
fn debug_spawn_ships(bc: &mut Battlescape, ship_queue: &mut ShipSpawnQueue) {
    let num = 1024;
    if bc.tick < num {
        let angle = (bc.tick % 16) as f32 * std::f32::consts::TAU / 16 as f32;
        let translation = na::UnitComplex::from_angle(angle) * na::vector![0.0, 10.0];
        let linvel = translation * -0.2;

        ship_queue.queue(ShipBuilder {
            ship_data_id: ShipDataId::BallShip,
            pos: na::Isometry2::new(translation, angle),
            linvel,
            angvel: (bc.tick % 10) as f32 * 0.5,
            team: bc.tick as u32,
        });
    }
}
