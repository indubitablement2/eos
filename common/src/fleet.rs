use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fleet {
    pub ships: Vec<Ship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub ship_data_id: ShipDataId,
}