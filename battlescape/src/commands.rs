use crate::{player_inputs::PlayerInput, PlayerId, ShipId, ShipUserId, TeamId, Tick};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattlescapeCommand {
    /// Spawn a ship for the specified team.
    /// Ship user data is entended for identifying this ship outside the battlescape.
    /// It is not used internally.
    SpawnShip {
        team_id: TeamId,
        ship_user_id: ShipUserId,
    },
    /// A player change its controlled ship.
    PlayerControlShip {
        player_id: PlayerId,
        ship_id: Option<ShipId>,
    },
    /// A player will automatically be removed if no player inputs are given in any preceding commands set.
    AddPlayer {
        player_id: PlayerId,
        team_id: TeamId,
    },
}

/// A set of commands needed to compute a particular tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeCommandsSet {
    pub tick: Tick,
    pub commands: Vec<BattlescapeCommand>,
    pub player_inputs: Vec<(PlayerId, PlayerInput)>,
}

pub struct CommandsSetBuffer(AHashMap<Tick, BattlescapeCommandsSet>);
impl CommandsSetBuffer {
    pub fn add_commands(&mut self, commands_set: BattlescapeCommandsSet) {
        self.0.entry(commands_set.tick).or_insert(commands_set);
    }
}
