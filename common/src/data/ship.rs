use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShipClass {
    Frigate,
    Destroyer,
    Cruiser,
    Capital,
    BattleStation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipBase {
    pub name: String,
    pub class: ShipClass,
    pub auto_combat_strenght: f32,
}
