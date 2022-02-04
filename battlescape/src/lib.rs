pub mod commands;
pub mod player_inputs;

#[macro_use]
extern crate log;
extern crate nalgebra as na;

use commands::BattlescapeCommand;
use player_inputs::PlayerInput;
use rand_xoshiro::Xoshiro128StarStar;
use rapier2d::prelude::*;
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BattlescapeCommandsQueue {
    commands: Vec<BattlescapeCommand>,
    num_command: Vec<u16>,
    #[serde(skip)]
    next_commands_index: usize,
}
impl BattlescapeCommandsQueue {
    // fn push_command(&mut self, command: BattlescapeCommand, tick: u32){
    //     debug_assert!(if let Some((last_tick, _)) = self.commands.last() {
    //         *last_tick <= tick
    //     } else {
    //         true
    //     });

    //     self.commands.push((tick, command));
    // }

    /// Return if there is any command queued for this tick.
    ///
    /// If a command is returned,
    /// this should be called again as there could be multiple commands for the same tick.
    fn get_next(&mut self, tick: u32) -> &[BattlescapeCommand] {
        let num_command = self.num_command[tick as usize] as usize;
        let slice = &self.commands[self.next_commands_index..num_command];
        self.next_commands_index += num_command;
        slice
    }
}

pub struct BattlescapeShip {
    player: u16,
    body_handle: RigidBodyHandle,
}

pub struct HumanPlayer {
    ship_control: Option<usize>,
    inputs: PlayerInput,
}
impl HumanPlayer {
    fn new() -> Self {
        Self { ship_control: None, inputs: Default::default() }
    }
}

pub struct AiPlayer {
    beep_boop: bool,
}
impl AiPlayer {
    fn new() -> Self {
        Self { beep_boop: true }
    }
}

pub enum PlayerType {
    HumanPlayer(HumanPlayer),
    AiPlayer(AiPlayer),
}

pub struct Player {
    player_type: PlayerType,
    team: u16,
    ships: Vec<u32>,
}
impl Player {
    fn new(player_type: PlayerType, team: u16) -> Self {
        Self {
            player_type,
            team,
            ships: Vec::new(),
        }
    }
}

pub struct Battlescape {
    bound: f32,
    tick: u32,

    battlescape_commands_queue: BattlescapeCommandsQueue,

    teams: Vec<Vec<u16>>,
    players: Vec<Player>,
    ships: Vec<BattlescapeShip>,

    rng: Xoshiro128StarStar,

    physics_pipeline: PhysicsPipeline,
    integration_parameters: IntegrationParameters,
    islands: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: JointSet,
    ccd_solver: CCDSolver,
}
impl Battlescape {
    pub fn update(&mut self) {
        self.apply_commands();

        self.physics_pipeline.step(
            &vector![0.0, 0.0],
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.ccd_solver,
            &(),
            &(),
        );

        self.tick += 1;
    }

    /// Apply commands for the current tick if any.
    fn apply_commands(&mut self) {
        for command in self.battlescape_commands_queue.get_next(self.tick) {
            match command {
                BattlescapeCommand::SpawnShip(cmd) => {
                    debug_assert!(self.players.len() > cmd.player as usize);

                    let body_handle = self.bodies.insert(RigidBodyBuilder::new_dynamic().build());
                    self.colliders.insert_with_parent(
                        ColliderBuilder::ball(1.0).build(),
                        body_handle,
                        &mut self.bodies,
                    );
                    self.ships.push(BattlescapeShip {
                        player: cmd.player,
                        body_handle,
                    });
                }
                BattlescapeCommand::AddPlayer(cmd) => {
                    let player_id = self.players.len() as u16;

                    let player_type = if cmd.human {
                        PlayerType::HumanPlayer(HumanPlayer::new())
                    } else {
                        PlayerType::AiPlayer(AiPlayer::new())
                    };

                    let team = cmd.team.unwrap_or_else(|| {
                        // Create a new team.
                        self.teams.push(vec![player_id]);
                        self.teams.len() as u16 - 1
                    });

                    let player = Player::new(player_type, team);

                    self.players.push(player);
                }
                BattlescapeCommand::PlayerInput(cmd) => {
                    let player = &self.players[cmd.player as usize];
                    // TODO
                }
            }
        }
    }
}
// impl Default for Battlescape {
//     fn default() -> Self {
//         Self {
//             bound: 512.0,
//             tick: Default::default(),
//             teams: Default::default(),
//             physics_pipeline: Default::default(),
//             integration_parameters: Default::default(),
//             islands: IslandManager::new(),
//             broad_phase: BroadPhase::new(),
//             narrow_phase: NarrowPhase::new(),
//             bodies: RigidBodySet::new(),
//             colliders: ColliderSet::new(),
//             joints: JointSet::new(),
//             ccd_solver: CCDSolver::new(),
//         }
//     }
// }

#[test]
fn a() {
    let f = -0.5f32;
    let u = (f * (u16::MAX / 2) as f32 + (u16::MAX / 2) as f32) as u16;
    let fc = u as f32 / (u16::MAX / 2) as f32 - 1.0;
    println!("{} {} {}", f, u, fc);
}