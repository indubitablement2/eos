use super::*;

#[derive(Debug, Clone, Default)]
pub struct BattlescapeEvents {
    pub add_ship: Vec<ShipId>,
    pub add_fleet: Vec<FleetId>,
}