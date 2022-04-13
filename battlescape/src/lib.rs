#![feature(slice_as_chunks)]

pub mod commands;
pub mod player_inputs;
pub mod state_init;

extern crate nalgebra as na;

use ahash::AHashSet;
use bincode::Options;
use commands::{BattlescapeCommand, BattlescapeCommandsSet};
use indexmap::IndexMap;
use na::UnitComplex;
use player_inputs::PlayerInput;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro128StarStar;
use rapier2d::prelude::*;
use serde::{self, Deserialize, Serialize};
use state_init::BattlescapeInitialState;

type ShipId = u32;
type TeamId = u8;
type ShipUserId = u32;
type PlayerId = u8;
type Tick = u32;

#[derive(Serialize, Deserialize)]
pub struct BattlescapeShip {
    user_id: ShipUserId,
    controller: Option<PlayerId>,
    body_handle: RigidBodyHandle,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    player_id: PlayerId,
    ship_control: Option<ShipId>,
    player_input: PlayerInput,
    team_id: TeamId,
}
impl Player {
    fn new(player_id: PlayerId, team_id: TeamId) -> Self {
        Self {
            player_id,
            ship_control: None,
            player_input: Default::default(),
            team_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Battlescape {
    bound: f32,
    /// The next tick to be processed when `update()` is called.
    tick: Tick,

    next_ship_id: ShipId,

    players: IndexMap<PlayerId, Player>,
    ships: IndexMap<ShipId, BattlescapeShip>,
    ships_team: IndexMap<TeamId, Vec<ShipId>>,

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
    joints: JointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}
impl Battlescape {
    pub fn new(battlescape_initial_state: BattlescapeInitialState) -> Self {
        Self {
            bound: battlescape_initial_state.bound,
            rng: Xoshiro128StarStar::seed_from_u64(battlescape_initial_state.seed),
            ..Default::default()
        }
    }

    /// Used internally. Exposed for debug purpose.
    pub fn spawn_ship(&mut self, team_id: TeamId, ship_user_id: ShipUserId) {
        // Add body.
        let body_handle = self.bodies.insert(RigidBodyBuilder::new_dynamic().build());
        self.colliders.insert_with_parent(
            ColliderBuilder::cuboid(0.5, 1.0).build(),
            body_handle,
            &mut self.bodies,
        );

        // Get a new ship id.
        let ship_id = self.next_ship_id;
        self.next_ship_id += 1;

        // Add ship.
        self.ships.insert(
            ship_id,
            BattlescapeShip {
                user_id: ship_user_id,
                controller: None,
                body_handle,
            },
        );

        // Add ship to team.
        self.ships_team.entry(team_id).or_default().push(ship_id);
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new()
            .serialize(self)
            .unwrap_or_default()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::DefaultOptions::new().deserialize(bytes)
    }

    pub fn update(&mut self, cmd_set: &BattlescapeCommandsSet) {
        self.apply_commands(cmd_set);
        self.ship_movement();

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

        self.query_pipeline.update(&self.islands, &self.bodies, &self.colliders);

        self.tick += 1;
    }

    /// Apply the commands for the current tick if available.
    fn apply_commands(&mut self, cmd_set: &BattlescapeCommandsSet) {
        // Handle the commands.
        for command in cmd_set.commands.iter() {
            match command {
                BattlescapeCommand::SpawnShip {
                    team_id,
                    ship_user_id,
                } => {
                    self.spawn_ship(*team_id, *ship_user_id);
                }
                BattlescapeCommand::PlayerControlShip { player_id, ship_id } => {
                    if let Some(player) = self.players.get_mut(player_id) {
                        // If the player was controlling a ship,
                        // set its controller back to None.
                        if let Some(ship_id) = &player.ship_control {
                            if let Some(prev_ship) = self.ships.get_mut(ship_id) {
                                prev_ship.controller = None;
                            }
                        }

                        // Set the player's controlled ship to None.
                        player.ship_control = None;

                        // Give the ship control to the player
                        if let Some(ship_id) = ship_id {
                            if let Some(ship) = self.ships.get_mut(ship_id) {
                                match ship.controller {
                                    Some(controlling_player_id) => {
                                        // Ship already controlled.
                                        log::info!(
                                            "Player {} can not take control of ship {} as it is already controlled by {}. Ignoring...",
                                            player_id, ship_id, controlling_player_id
                                        );
                                    }
                                    None => {
                                        ship.controller = Some(*player_id);
                                        player.ship_control = Some(*ship_id);
                                    }
                                }
                            } else {
                                // Ship not found.
                                log::warn!(
                                    "Player {} requested control of ship {}, but it does not exist. Ignoring...", 
                                    player_id, ship_id
                                );
                            }
                        }
                    } else {
                        // Player not found.
                        log::warn!(
                            "Player {} requested to control a ship, but this player does not exist.", 
                            player_id
                        );
                    }
                }
                BattlescapeCommand::AddPlayer { player_id, team_id } => {
                    match self
                        .players
                        .insert(*player_id, Player::new(*player_id, *team_id))
                    {
                        Some(_) => {
                            log::info!("Player {} was updated.", player_id);
                        }
                        None => {
                            log::info!("Player {} was added.", player_id);
                        }
                    }
                }
            }
        }

        // Set the player's new inputs.
        let mut inactive_players: AHashSet<PlayerId> = self.players.keys().copied().collect();
        for (player_id, player_input) in cmd_set.player_inputs.iter() {
            if let Some(player) = self.players.get_mut(player_id) {
                player.player_input = *player_input;
                inactive_players.remove(player_id);
            } else {
                // Player not found.
                log::warn!(
                    "Commands set has input for player {}, but it does not exist. Ignoring...",
                    player_id
                );
            }
        }

        // Remove inactive players.
        for player_id in inactive_players.iter() {
            self.players.shift_remove(player_id);
            log::info!("Player {} is inactive and was removed.", player_id);
        }
    }

    fn ship_movement(&mut self) {
        for (ship_id, ship) in self.ships.iter_mut() {
            if let Some(player_id) = ship.controller {
                if let Some(player) = self.players.get(&player_id) {
                    if let Some(body) = self.bodies.get_mut(ship.body_handle) {
                        // Apply wish dir.
                        let wish_dir = player.player_input.get_wish_dir();
                        let force = UnitComplex::new(wish_dir.angle) * vector![wish_dir.force, 0.0];
                        body.apply_force(force, true);

                        // Apply wish rot.
                        match player.player_input.get_wish_rot() {
                            player_inputs::WishRot::Relative(f) => {
                                body.apply_torque(f, true);
                            }
                            player_inputs::WishRot::FaceWorldPositon(x, y) => {
                                let wish_angle_cart =
                                    (vector![x, y] - *body.translation()).normalize();
                                let wish_angle = UnitComplex::from_cos_sin_unchecked(
                                    wish_angle_cart.x,
                                    wish_angle_cart.y,
                                );
                                let current_angle = body.rotation().angle_to(&wish_angle);
                                body.apply_torque(current_angle.signum(), true);
                            }
                        }
                    }
                } else {
                    // Controlling player not found.
                    ship.controller = None;
                    log::info!(
                        "Ship {} is controlled by player {}, but it is not found. Removing control...", 
                        ship_id, player_id
                    );
                }
            } else {
                // TODO: Ship AI.
            }
        }
    }

    /// Get the next tick that will be processed when `update()` is called.
    #[must_use]
    pub fn tick(&self) -> u32 {
        self.tick
    }
}
impl Default for Battlescape {
    fn default() -> Self {
        Self {
            bound: 512.0,
            tick: Default::default(),
            physics_pipeline: Default::default(),
            integration_parameters: Default::default(),
            islands: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            players: Default::default(),
            ships: Default::default(),
            rng: Xoshiro128StarStar::seed_from_u64(1377),
            next_ship_id: 0,
            ships_team: Default::default(),
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
