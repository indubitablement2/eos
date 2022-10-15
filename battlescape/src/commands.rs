use crate::player_inputs::PlayerInput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnShip {
    pub player_id: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPlayer {
    /// If this player will be added to an existing team or create its own.
    pub team_id: Option<u16>,
    pub human: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPlayerInput {
    pub player_id: u16,
    pub player_input: PlayerInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerControlShip {
    pub player_id: u16,
    pub ship_idx: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattlescapeCommand {
    // SpawnShip(SpawnShip),
    // AddPlayer(AddPlayer),
    SetPlayerInput(SetPlayerInput),
    // PlayerControlShip(PlayerControlShip),
}
