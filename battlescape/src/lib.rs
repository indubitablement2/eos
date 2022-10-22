#![feature(slice_as_chunks)]
#![feature(duration_consts_float)]

pub mod commands;
pub mod hull;
pub mod physics;
pub mod player_inputs;
mod schedule;
pub mod state_init;

extern crate nalgebra as na;

use commands::BattlescapeCommand;
use rapier2d::data::{Arena, Index};
use schedule::*;
use state_init::BattlescapeInitialState;
use std::time::Duration;
use utils::rand::RNG;

pub use ahash::AHashMap;
pub use data::hull_data::*;
pub use data::*;
pub use hull::*;
pub use physics::*;
pub use rand::prelude::*;
pub use rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use smallvec::{smallvec, SmallVec};

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub bound: f32,
    pub tick: u64,
    rng: RNG,
    pub physics: Physics,

    pub hulls: Arena<Hull>,
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
            hulls: Default::default(),
        }
    }

    pub fn step(&mut self, cmds: &[BattlescapeCommand]) {
        apply_commands::apply_commands(self, cmds);
        debug_spawn_ships(self);
        self.physics.step();
        // TODO: Handle events.
        self.tick += 1;
    }

    pub fn save(&self) -> Vec<u8> {
        // Afaik this can not fail.
        bincode::Options::serialize(bincode::DefaultOptions::new(), self).unwrap()
    }

    pub fn load(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes)
    }

    pub fn spawn_hull(&mut self, hull_builder: HullBuilder) -> Index {
        let hull_data = hull_data(hull_builder.hull_data_id);

        let rb = self.physics.add_body(
            hull_builder.pos,
            hull_builder.linvel,
            hull_builder.angvel,
            hull_data.shape.to_shared_shape(),
            hull_data.density,
            None,
            hull_builder.team,
            None,
            hull_data.groups,
        );

        // TODO: Add joined childs.
        let childs = Childs::new();

        self.hulls.insert(Hull::new(hull_builder, rb, childs, None))
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[deprecated]
fn debug_spawn_ships(bc: &mut Battlescape) {
    if bc.tick != 0 {
        return;
    }

    let num = 8;
    for i in 0..num {
        let angle = i as f32 * std::f32::consts::TAU / num as f32;
        let translation = na::UnitComplex::from_angle(angle) * na::vector![0.0, 10.0];
        let linvel = translation * -0.1;

        bc.spawn_hull(HullBuilder {
            hull_data_id: HullDataId(i % 2),
            pos: na::Isometry2::new(translation, angle),
            linvel,
            angvel: 0.0,
            team: i as u32,
        });
    }
}
