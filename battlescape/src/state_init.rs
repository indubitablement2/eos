use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeInitialState {
    pub bound: f32,
    pub seed: u64,
}
