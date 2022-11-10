use super::*;
use common::fleet::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeFleet {
    pub original_fleet: Fleet,
    pub available_ships: AHashMap<usize, ShipId>,
    pub team: Team,
}
