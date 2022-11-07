#![feature(slice_as_chunks)]
#![feature(duration_consts_float)]

pub mod commands;
pub mod hull;
pub mod physics;
pub mod player_inputs;
mod schedule;
pub mod state_init;
pub mod hull_queue;

extern crate nalgebra as na;

use commands::BattlescapeCommand;
use hull_queue::HullSpawnQueue;
use schedule::*;
use state_init::BattlescapeInitialState;
use std::time::Duration;
use utils::rand::RNG;
use indexmap::IndexMap;

pub use ahash::AHashMap;
pub use data::hull_data::*;
pub use data::*;
pub use hull::*;
pub use physics::*;
pub use rand::prelude::*;
pub use rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use smallvec::{smallvec, SmallVec};
pub use utils::packed_map::*;

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub bound: f32,
    pub tick: u64,
    rng: RNG,
    pub physics: Physics,

    next_hull_id: u32,
    // TODO: use index map
    pub hulls: IndexMap<HullId, Hull, ahash::RandomState>,
}
impl Battlescape {
    pub const TICK_DURATION: Duration = Duration::from_millis(50);
    pub const TICK_DURATION_SEC: f32 = Self::TICK_DURATION.as_secs_f32();
    pub const TICK_DURATION_MS: u32 = Self::TICK_DURATION.as_millis() as u32;

    pub fn new(battlescape_initial_state: BattlescapeInitialState) -> Self {
        Self {
            bound: battlescape_initial_state.bound,
            rng: RNG::seed_from_u64(battlescape_initial_state.seed),
            tick: 0,
            physics: Default::default(),
            next_hull_id: 0,
            hulls: Default::default(),
        }
    }

    pub fn step(&mut self, cmds: &[BattlescapeCommand]) {
        let mut queue = HullSpawnQueue::new(self);

        apply_commands::apply_commands(self, cmds);
        debug_spawn_ships(self, &mut queue);
        self.physics.step();
        // TODO: Handle events.

        queue.process(self);
        
        self.tick += 1;
    }

    pub fn save(&self) -> Vec<u8> {
        // Afaik this can not fail.
        bincode::Options::serialize(bincode::DefaultOptions::new(), self).unwrap()
    }

    pub fn load(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes)
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[deprecated]
fn debug_spawn_ships(bc: &mut Battlescape, queue: &mut HullSpawnQueue) {
    // bc.spawn_hull(HullBuilder {
    //     hull_data_id: HullDataId(1),
    //     pos: na::Isometry2::new(na::vector![0.5, 0.5], 0.0),
    //     linvel: na::Vector2::zeros(),
    //     angvel: 0.0,
    //     team: 0,
    // });
    // bc.spawn_hull(HullBuilder {
    //     hull_data_id: HullDataId(1),
    //     pos: na::Isometry2::new(na::vector![-0.5, 0.5], 0.0),
    //     linvel: na::Vector2::zeros(),
    //     angvel: 0.0,
    //     team: 0,
    // });
    // bc.spawn_hull(HullBuilder {
    //     hull_data_id: HullDataId(1),
    //     pos: na::Isometry2::new(na::vector![0.5, -0.5], 0.0),
    //     linvel: na::Vector2::zeros(),
    //     angvel: 0.0,
    //     team: 0,
    // });
    // bc.spawn_hull(HullBuilder {
    //     hull_data_id: HullDataId(1),
    //     pos: na::Isometry2::new(na::vector![-0.5, -0.5], 0.0),
    //     linvel: na::Vector2::zeros(),
    //     angvel: 0.0,
    //     team: 0,
    // });

    let iter = 1;
    let num = 2048;
    let i = bc.tick / iter;
    if bc.tick % iter == 0 && i < num {
        log::debug!("{}", i);

        let angle = i as f32 * std::f32::consts::TAU / 16 as f32;
        let translation = na::UnitComplex::from_angle(angle) * na::vector![0.0, 10.0];
        let linvel = translation * -0.2;
        
        queue.queue(HullBuilder {
            hull_data_id: HullDataId((i % 2) as u32),
            pos: na::Isometry2::new(translation, angle),
            linvel,
            angvel: (i % 10) as f32 * 0.5,
            team: i as u32,
        });
    }
}
