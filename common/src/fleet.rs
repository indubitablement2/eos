use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fleet {
    /// If a client own this fleet. Otherwise the fleet is owned by an ai.
    pub owner: Option<ClientId>,
    pub ships: Vec<Ship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub ship_data_id: ShipDataId,
}