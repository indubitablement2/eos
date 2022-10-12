#![feature(slice_as_chunks)]

pub mod commands;
pub mod hull;
pub mod physics;
pub mod player_inputs;
pub mod replay;
mod schedule;
pub mod ship;
pub mod state_init;

extern crate nalgebra as na;

pub use ahash::AHashMap;
pub use hull::*;
pub use physics::Physics;
pub use rand::prelude::*;
pub use rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use ship::*;
pub use smallvec::SmallVec;

use commands::BattlescapeCommand;
use player_inputs::PlayerInput;
use rand_xoshiro::Xoshiro256StarStar;
use schedule::*;
use state_init::BattlescapeInitialState;
use rapier2d::data::Arena;

#[derive(Serialize, Deserialize)]
pub struct BattlescapeShip {
    player_id: u16,
    controlled: bool,
    body_handle: RigidBodyHandle,
}

#[derive(Serialize, Deserialize)]
pub struct HumanPlayer {
    ship_control: Option<Vec<u32>>,
    player_input: PlayerInput,
}
impl HumanPlayer {
    fn new() -> Self {
        Self {
            ship_control: None,
            player_input: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AiPlayer {
    beep_boop: bool,
}
impl AiPlayer {
    fn new() -> Self {
        Self { beep_boop: true }
    }
}

#[derive(Serialize, Deserialize)]
pub enum PlayerType {
    HumanPlayer(HumanPlayer),
    AiPlayer(AiPlayer),
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    player_type: PlayerType,
    team_id: u16,
    ship_idx: Vec<u32>,
}
impl Player {
    fn new(player_type: PlayerType, team_id: u16) -> Self {
        Self {
            player_type,
            team_id,
            ship_idx: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    bound: f32,
    tick: u64,
    rng: rand_xoshiro::Xoshiro256StarStar,
    physics: Physics,

    hulls: Arena<Hull>,
    ships: Arena<Ship>,
}
impl Battlescape {
    pub fn new(battlescape_initial_state: BattlescapeInitialState) -> Self {
        
        Self {
            bound: battlescape_initial_state.bound,
            rng: Xoshiro256StarStar::seed_from_u64(battlescape_initial_state.seed),
            tick: 0,
            physics: Default::default(),
            hulls: Default::default(),
            ships: Default::default(),
        }
    }

    pub fn step(&mut self, cmds: &[BattlescapeCommand]) {
        apply_commands::apply_commands(self, cmds);
        self.physics.step();
        // TODO: Handle events.
        self.tick += 1;
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::Options::serialize(bincode::DefaultOptions::new(), self).unwrap_or_default()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::Options::deserialize(bincode::DefaultOptions::new(), bytes)
    }

    pub fn checksum(&self) -> u32 {
        crc32fast::hash(&self.serialize())
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
