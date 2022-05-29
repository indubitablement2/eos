use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Colony {
    pub faction: Option<FactionId>,
    pub guards: Vec<FleetId>,
    pub population: u64,
}
