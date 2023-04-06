use super::*;
use crate::battlescape::entity::EntityCondition;
use godot::{engine::Texture2D, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub ship_data_id: ShipDataId,
    pub condition: EntityCondition,
}

#[derive(Debug)]
pub struct ShipData {
    pub display_name: String,
    /// Sprite2D
    pub texture: Gd<Texture2D>,
    pub entity_data_id: EntityDataId,
}

impl Default for ShipData {
    fn default() -> Self {
        Self {
            display_name: Default::default(),
            texture: load::<Texture2D>("res://textures/error.png"),
            entity_data_id: Default::default(),
        }
    }
}
