use super::*;
use godot::engine::Texture2D;
use godot::prelude::*;

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
    pub display_name: String,
    pub texture: Gd<Texture2D>,
    pub entity_data_id: EntityDataId,
}
