use crate::player_inputs::PlayerInput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnShipBattlescapeCommand {
    pub player_id: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPlayerBattlescapeCommand {
    /// If this player will be added to an existing team or create its own.
    pub team_id: Option<u16>,
    pub human: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInputBattlescapeCommand {
    pub player_id: u16,
    pub player_input: PlayerInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerControlShipBattlescapeCommand {
    pub player_id: u16,
    pub ship_idx: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattlescapeCommand {
    SpawnShip(SpawnShipBattlescapeCommand),
    AddPlayer(AddPlayerBattlescapeCommand),
    PlayerInput(PlayerInputBattlescapeCommand),
    PlayerControlShip(PlayerControlShipBattlescapeCommand),
}
