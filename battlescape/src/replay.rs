use crate::state_init::BattlescapeInitialState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeOutcome {
    pub ship_id: u32,
    pub hp: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeReplay {
    pub battlescape_initial_state: BattlescapeInitialState,
    // pub battlescape_commands_queue: BattlescapeCommandsQueue,
    pub hashes: Vec<u64>,
    pub outcome: Vec<BattlescapeOutcome>,
}
