use serde::{Serialize, Deserialize};
use crate::idx::{ShipBaseId, WeaponBaseId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipState {
    pub hp: f32,
    pub state: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipInfo {
    pub ship: ShipBaseId,
    pub weapons: Vec<WeaponBaseId>,
}
impl ShipInfo {
    pub fn compute_auto_combat_strenght(&self, state: &ShipState, bases: &Bases) -> f32 {
        bases.ships[self.ship].auto_combat_strenght * state.state
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaponSize {
    Light,
    Medium,
    Heavy,
    Experimental
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponBase {
    pub name: String,
    pub size: WeaponSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bases {
    pub ships: Vec<ShipBase>,
    pub weapons: Vec<WeaponBase>,
}
impl Default for Bases {
    fn default() -> Self {
        Self {
            ships: vec![
                ShipBase {
                    name: "Frig".to_string(),
                    class: ShipClass::Frigate,
                    auto_combat_strenght: 100.0,
                },
                ShipBase {
                    name: "Destro".to_string(),
                    class: ShipClass::Destroyer,
                    auto_combat_strenght: 300.0,
                },
            ],
            weapons: vec![
                WeaponBase { name: "Pee shooter".to_string(), size: WeaponSize::Light },
                WeaponBase { name: "Potato laucher".to_string(), size: WeaponSize::Medium },
            ]
        }
    }
}
