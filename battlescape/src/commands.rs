use crate::{player_inputs::PlayerInput, state_init::BattlescapeInitialState};
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
pub struct PlayerControlShip {
    pub player_id: u16,
    pub ship_idx: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPlayerInput {
    pub player_id: u16,
    pub player_input: PlayerInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattlescapeCommand {
    // SpawnShip(SpawnShip),
    // AddPlayer(AddPlayer),
    // PlayerControlShip(PlayerControlShip),
    SetPlayerInput(SetPlayerInput),
}

/// When Some(Vec<u8>), force load a jump point before applying the cmds to stay deteministic.
pub type Cmds = (Vec<BattlescapeCommand>, Option<Vec<u8>>);

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Replay {
    #[serde(skip)]
    remaining: Vec<(u64, Cmds)>,
    pub initial_state: BattlescapeInitialState,
    /// The cmds where the index is the tick.
    pub cmds: Vec<Cmds>,
}
impl Replay {
    pub fn push_cmds(&mut self, tick: u64, cmds: Cmds) {
        let mut next_tick = self.cmds.len() as u64;
        if tick == next_tick {
            self.cmds.push(cmds);

            // Maybe we can use some of the remaining ticks now.
            next_tick = self.cmds.len() as u64;
            loop {
                if let Some((tick, _)) = self.remaining.first() {
                    if *tick == next_tick {
                        self.cmds.push(self.remaining.remove(0).1);
                    } else {
                        // Can not use that tick yet.
                        break;
                    }
                } else {
                    // No more remaining tick.
                    break;
                }
            }
        } else if tick < next_tick {
            // Already have that tick.
            log::debug!("Replay received tick {} twice while waiting for tick {}. Ignoring...", tick, next_tick);
        } else {
            match self.remaining.binary_search_by(|probe| {
                probe.0.cmp(&tick)
            }) {
                Ok(_) => {
                    // Already have that tick.
                    log::debug!("Replay received tick {} twice while waiting for tick {}. Ignoring...", tick, next_tick);
                }
                Err(i) =>  {
                    self.remaining.insert(i, (tick, cmds));
                }
            }
        }
    }
}