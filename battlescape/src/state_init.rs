use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BattlescapeInitialState {
    pub bound: f32,
    pub seed: u64,

}