#![feature(slice_as_chunks)]

pub mod commands;
pub mod hull;
pub mod physics;
pub mod player_inputs;
pub mod replay;
mod schedule;
pub mod state_init;

extern crate nalgebra as na;

pub use ahash::AHashMap;
pub use physics::Physics;
pub use rand::prelude::*;
pub use rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use smallvec::SmallVec;

use commands::BattlescapeCommand;
use player_inputs::PlayerInput;
use rand_xoshiro::Xoshiro256StarStar;
use schedule::*;
use state_init::BattlescapeInitialState;

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
}
impl Battlescape {
    pub fn new(battlescape_initial_state: BattlescapeInitialState) -> Self {
        Self {
            bound: battlescape_initial_state.bound,
            rng: Xoshiro256StarStar::seed_from_u64(battlescape_initial_state.seed),
            tick: 0,
            physics: Default::default(),
        }
    }

    pub fn step(&mut self, cmds: &[BattlescapeCommand]) {
        apply_commands::apply_commands(self, cmds);
        self.ship_movement();
        self.physics.step();
        // TODO: Events.
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

    fn ship_movement(&mut self) {
        // for ship in self.ships.values_mut() {
        //     if ship.controlled {
        //         let human_player = if let PlayerType::HumanPlayer(human_player) =
        //             &self.players[ship.player_id as usize].player_type
        //         {
        //             human_player
        //         } else {
        //             continue;
        //         };

        //         if let Some(body) = self.bodies.get_mut(ship.body_handle) {
        //             // Apply wish dir.
        //             let wish_dir = human_player.player_input.get_wish_dir();
        //             let force = UnitComplex::new(wish_dir.0) * vector![wish_dir.1, 0.0];
        //             // body.apply_force(force, true);

        //             // Apply wish rot.
        //             match human_player.player_input.get_wish_rot() {
        //                 player_inputs::WishRot::Relative(f) => {
        //                     // body.apply_torque(f, true);
        //                 }
        //                 player_inputs::WishRot::FaceWorldPositon(x, y) => {
        //                     let wish_angle_cart = (vector![x, y] - *body.translation()).normalize();
        //                     let wish_angle = UnitComplex::from_cos_sin_unchecked(
        //                         wish_angle_cart.x,
        //                         wish_angle_cart.y,
        //                     );
        //                     let current_angle = body.rotation().angle_to(&wish_angle);
        //                     // body.apply_torque(current_angle.signum(), true);
        //                 }
        //             }
        //         }
        //     } else {
        //     }
        // }
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
