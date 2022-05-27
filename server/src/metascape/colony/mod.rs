use super::*;

#[derive(Debug, Clone, Fields, Columns, Components)]
pub struct Colony {
    pub faction: Option<FactionId>,
    pub guards: Vec<FleetId>,
    pub population: u64,
}
