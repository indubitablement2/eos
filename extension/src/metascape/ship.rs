use super::battlescape::entity::EntityDataTransient;
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub ship_data_id: ShipDataId,
    /// In absolute value 0..1
    pub hull: f32,
    /// In absolute value 0..1
    pub armor: f32,
    pub readiness: f32,
}

#[derive(Debug)]
pub struct ShipData {
    pub entity_data_id: EntityDataId,
}

/// Ship read from file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipDataTransient {
    pub entity: EntityDataTransient,
}
impl ShipDataTransient {
    pub fn to_ship_data(self, data: &mut Data) -> ShipData {
        let entity_data_id = data.add_entity_data(self.entity.to_entity_data());
        ShipData { entity_data_id }
    }
}
