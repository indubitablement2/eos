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

use std::time::Duration;
pub use ahash::AHashMap;
pub use hull::*;
pub use physics::*;
pub use rand::prelude::*;
pub use rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use ship::*;
pub use smallvec::{smallvec, SmallVec};

use commands::BattlescapeCommand;
use player_inputs::PlayerInput;
use rapier2d::data::{Arena, Index};
use schedule::*;
use state_init::BattlescapeInitialState;
use utils::rand::RNG;

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    pub bound: f32,
    pub tick: u64,
    rng: RNG,
    pub physics: Physics,

    pub hulls: Arena<Hull>,
    pub ships: Arena<Ship>,
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
            ships: Default::default(),
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

    pub fn spanw_ship(&mut self, ship_builder: ShipBuilder) -> Index {
        let ship_data = &BATTLESCAPE_DATA.ships[ship_builder.ship_data_index];

        let mut ignore_rb = None;
        let mut membership = PhysicsGroup::SHIP;
        let mut hulls_index = SmallVec::with_capacity(ship_data.hulls_data_index.len());
        // Create each hull.
        for &hull_data_index in ship_data.hulls_data_index.iter() {
            let hull_data = &BATTLESCAPE_DATA.hulls[hull_data_index];

            let rb_handle = self.physics.add_body(
                ship_builder.pos,
                ship_builder.linvel,
                ship_builder.angvel,
                hull_data.shape.to_shared_shape(),
                hull_data.density,
                ignore_rb,
                Some(ship_builder.team),
                None,
                membership,
                PhysicsGroup::DEFAULT_SHIP_FILTER,
            );

            let hull_index = self.hulls.insert(Hull {
                hull_data_index,
                rb: rb_handle,
                defence: hull_data.defence,
            });

            hulls_index.push(hull_index);

            if ignore_rb.is_none() {
                ignore_rb = Some(rb_handle);
            }
            membership = PhysicsGroup::SHIP_AUXILIARY;
        }

        // Insert the ship.
        self.ships.insert(Ship {
            ship_data_index: ship_builder.ship_data_index,
            mobility: ship_data.mobility,
            hulls_index,
        })
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
    log::debug!("spawned ship");
    let num = 8;
    for i in 0..num {
        let angle = i as f32 * std::f32::consts::TAU / num as f32;
        let translation = na::UnitComplex::from_angle(angle) * na::vector![0.0, 10.0];
        let linvel = translation * -0.1;

        bc.spanw_ship(ShipBuilder {
            ship_data_index: 0,
            pos: na::Isometry2::new(translation, angle),
            linvel,
            angvel: 0.0,
            team: i as u32,
        });
    }
}
