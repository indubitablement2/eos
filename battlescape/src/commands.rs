use super::*;
use crate::state_init::BattlescapeInitialState;
use common::fleet::Fleet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnShip {
    pub player_id: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetClientControl {
    pub client_id: ClientId,
    pub ship_id: Option<ShipId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetClientInput {
    pub client_id: ClientId,
    pub inputs: bc_client::PlayerInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFleet {
    pub fleet_id: FleetId,
    pub fleet: Fleet,
    /// This should be an existing team.
    /// If `None`, create a new team.
    pub team: Option<Team>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattlescapeCommand {
    SetClientControl(SetClientControl),
    SetClientInput(SetClientInput),
    AddFleet(AddFleet),
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct FullCmds {
    /// When `Some(data)`, force load a jump point before applying the cmds to stay deteministic.
    ///
    /// There is also a checksum of the data.
    /// Should be the same as applying each cmds before the jump point.
    pub jump_point: Option<(Vec<u8>, u32)>,
    /// The cmds to apply this tick after applying the jump point (if any).
    pub cmds: Vec<BattlescapeCommand>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Replay {
    #[serde(skip)]
    remaining: Vec<(u64, FullCmds)>,
    pub initial_state: BattlescapeInitialState,
    /// The cmds to apply to a tick. index == tick.
    pub cmds: Vec<FullCmds>,
    /// Checksum taken after applying the final cmds.
    pub final_checksum: u32,
}
impl Replay {
    pub fn push_cmds(&mut self, tick: u64, cmds: FullCmds) {
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
            log::debug!(
                "Replay received tick {} twice while waiting for tick {}. Ignoring...",
                tick,
                next_tick
            );
        } else {
            match self.remaining.binary_search_by(|probe| probe.0.cmp(&tick)) {
                Ok(_) => {
                    // Already have that tick.
                    log::debug!(
                        "Replay received tick {} twice while waiting for tick {}. Ignoring...",
                        tick,
                        next_tick
                    );
                }
                Err(i) => {
                    self.remaining.insert(i, (tick, cmds));
                }
            }
        }
    }

    pub fn get_cmds(&self, tick: u64) -> Option<&FullCmds> {
        self.cmds.get(tick as usize)
    }
}
