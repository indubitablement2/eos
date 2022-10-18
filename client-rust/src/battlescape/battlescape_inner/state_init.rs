use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BattlescapeInitialState {
    pub bound: f32,
    pub seed: u64,
}
impl Default for BattlescapeInitialState {
    fn default() -> Self {
        Self {
            bound: 100.0,
            seed: 1337,
        }
    }
}