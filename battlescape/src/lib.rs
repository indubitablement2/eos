#![feature(slice_as_chunks)]

pub mod commands;
pub mod player_inputs;
pub mod replay;
pub mod state_init;

// #[macro_use]
// extern crate log;
extern crate nalgebra as na;

use bincode::Options;
use commands::BattlescapeCommand;
use na::UnitComplex;
use player_inputs::PlayerInput;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro128StarStar;
use rapier2d::prelude::*;
use replay::BattlescapeReplay;
use serde::{self, Deserialize, Serialize};
use state_init::BattlescapeInitialState;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BattlescapeCommandsQueue {
    commands: Vec<Vec<BattlescapeCommand>>,
}
impl BattlescapeCommandsQueue {
    pub fn push_commands(&mut self, commands: &[BattlescapeCommand], tick: u32) {
        if self.commands.len() as u32 <= tick {
            self.commands.resize(tick as usize + 1, Vec::new());
        }

        self.commands[tick as usize].extend_from_slice(commands);
    }

    /// Return the commands queued for this tick if any.
    fn get_next(&mut self, tick: u32) -> Option<&Vec<BattlescapeCommand>> {
        self.commands.get(tick as usize)
    }
}

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
    tick: u32,

    battlescape_commands_queue: BattlescapeCommandsQueue,

    teams: Vec<Vec<u16>>,
    players: Vec<Player>,

    next_ship_id: u32,
    // ships: IndexMap<u32, BattlescapeShip>,

    rng: Xoshiro128StarStar,

    #[serde(skip)]
    physics_pipeline: PhysicsPipeline,
    #[serde(skip)]
    integration_parameters: IntegrationParameters,
    islands: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    // joints: JointSet,
    ccd_solver: CCDSolver,
}
impl Battlescape {
    pub fn new(battlescape_initial_state: BattlescapeInitialState) -> Self {
        Self {
            bound: battlescape_initial_state.bound,
            rng: Xoshiro128StarStar::seed_from_u64(battlescape_initial_state.seed),
            ..Default::default()
        }
    }

    pub fn new_from_replay(battlescape_replay: &BattlescapeReplay) -> Self {
        let mut battlescape = Battlescape::new(battlescape_replay.battlescape_initial_state);
        battlescape.battlescape_commands_queue = battlescape_replay.battlescape_commands_queue.to_owned();
        battlescape
    }

    pub fn update(&mut self) {
        self.apply_commands();
        self.ship_movement();

        // self.physics_pipeline.step(
        //     &vector![0.0, 0.0],
        //     &self.integration_parameters,
        //     &mut self.islands,
        //     &mut self.broad_phase,
        //     &mut self.narrow_phase,
        //     &mut self.bodies,
        //     &mut self.colliders,
        //     &mut self.joints,
        //     &mut self.ccd_solver,
        //     &(),
        //     &(),

        // );

        self.tick += 1;
    }

    pub fn push_commands(&mut self, commands: &[BattlescapeCommand], tick: u32) {
        assert!(
            self.tick <= tick,
            "Can not push commands for a previous tick."
        );
        self.battlescape_commands_queue
            .push_commands(commands, tick)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new()
            .serialize(self)
            .unwrap_or_default()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::DefaultOptions::new().deserialize(bytes)
    }

    // /// Get a reference to the battlescape's ships.
    // pub fn ships(&self) -> &IndexMap<u32, BattlescapeShip> {
    //     &self.ships
    // }

    /// Apply commands for the current tick if any.
    fn apply_commands(&mut self) {
        let commands = if let Some(commands) = self.battlescape_commands_queue.get_next(self.tick) {
            commands
        } else {
            return;
        };

        for command in commands {
            match command {
                BattlescapeCommand::SpawnShip(cmd) => {
                    debug_assert!(self.players.len() > cmd.player_id as usize);

                    // let body_handle = self.bodies.insert(RigidBodyBuilder::new_dynamic().build());
                    // self.colliders.insert_with_parent(
                    //     ColliderBuilder::cuboid(0.5, 1.0).build(),
                    //     body_handle,
                    //     &mut self.bodies,
                    // );

                    // let ship_id = self.next_ship_id;
                    // self.next_ship_id += 1;

                    // self.ships.insert(
                    //     ship_id,
                    //     BattlescapeShip {
                    //         player_id: cmd.player_id,
                    //         controlled: false,
                    //         body_handle,
                    //     },
                    // );
                }
                BattlescapeCommand::AddPlayer(cmd) => {
                    let player_id = self.players.len() as u16;

                    let player_type = if cmd.human {
                        PlayerType::HumanPlayer(HumanPlayer::new())
                    } else {
                        PlayerType::AiPlayer(AiPlayer::new())
                    };

                    let team = cmd.team_id.unwrap_or_else(|| {
                        // Create a new team.
                        self.teams.push(vec![player_id]);
                        self.teams.len() as u16 - 1
                    });

                    let player = Player::new(player_type, team);

                    self.players.push(player);
                }
                BattlescapeCommand::PlayerInput(cmd) => {
                    let player = &mut self.players[cmd.player_id as usize];
                    let player = if let PlayerType::HumanPlayer(player) = &mut player.player_type {
                        player
                    } else {
                        continue;
                    };
                    player.player_input = cmd.player_input;
                }
                BattlescapeCommand::PlayerControlShip(cmd) => {
                    let human_player =
                        if let Some(player) = self.players.get_mut(cmd.player_id as usize) {
                            if let PlayerType::HumanPlayer(human_player) = &mut player.player_type {
                                human_player
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        };

                    // Set the previously controlled ships back to false.
                    if let Some(ship_idx) = &human_player.ship_control {
                        for ship_id in ship_idx.iter() {
                            // self.ships[*ship_id as usize].controlled = false;
                        }
                    }

                    // // Filter out ships that are not owned by the player.
                    // let ship_idx = cmd.ship_idx.as_ref().map(|ship_idx| {
                    //     ship_idx
                    //         .iter()
                    //         .filter(|&&ship_id| {
                    //             if let Some(ship) = self.ships.get_mut(&ship_id) {
                    //                 if ship.player_id == cmd.player_id {
                    //                     ship.controlled = true;
                    //                     true
                    //                 } else {
                    //                     false
                    //                 }
                    //             } else {
                    //                 false
                    //             }
                    //         })
                    //         .copied()
                    //         .collect()
                    // });

                    // human_player.ship_control = ship_idx;
                }
            }
        }
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
        Self {
            bound: 512.0,
            tick: Default::default(),
            teams: Default::default(),
            physics_pipeline: Default::default(),
            integration_parameters: Default::default(),
            islands: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            // joints: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            battlescape_commands_queue: Default::default(),
            players: Default::default(),
            // ships: Default::default(),
            rng: Xoshiro128StarStar::seed_from_u64(1377),
            next_ship_id: Default::default(),
        }
    }
}

pub trait HashBattlescape {
    fn simple_hash(&self) -> u64;
}
impl HashBattlescape for [u8] {
    fn simple_hash(&self) -> u64 {
        let (chunk, remainder) = self.as_chunks();
        chunk
            .iter()
            .fold(0u64, |acc, x| acc.wrapping_add(u64::from_le_bytes(*x)))
            .wrapping_add(remainder.iter().fold(0u64, |acc, x| acc + *x as u64))
    }
}

#[test]
fn test_hash() {
    let mut bc = Battlescape::default();
    bc.colliders.insert(ColliderBuilder::ball(1.0).build());
    bc.colliders.insert(ColliderBuilder::ball(2.0).build());
    let first = bc.serialize().simple_hash();
    let mut bc = Battlescape::default();
    bc.colliders.insert(ColliderBuilder::ball(2.0).build());
    bc.colliders.insert(ColliderBuilder::ball(1.0).build());
    let second = bc.serialize().simple_hash();
    let second_second = bc.serialize().simple_hash();

    println!(
        "{} - {} / {} - dif: {}",
        first,
        second,
        second_second,
        (first as i128 - second as i128).abs()
    );
    assert_ne!(first, second);
    assert_eq!(second, second_second);
}

// #[test]
// fn a() {
//     use glam::Vec2;
//     let b = UnitComplex::rotation_between(&vector![0.0, -1.0], &vector![1.0, -0.0]);
//     let a = vector![0.0, -1.0].angle(&vector![-10.0, 0.0]);
//     println!("{}", b);
// }
