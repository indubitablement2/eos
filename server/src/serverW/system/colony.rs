use common::idx::*;

#[derive(Debug, Clone)]
pub struct Colony {
    pub faction: Option<FactionId>,
    pub guards: Vec<FleetId>,
    pub population: u64,
}
