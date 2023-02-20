use super::*;
use godot::{engine::Texture2D, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub ship_data_id: ShipDataId,
    pub entity_condition: EntityCondition,
}

/// In absolute value 0..1
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntityCondition {
    pub hull: f32,
    pub armor: f32,
    pub readiness: f32,
}
impl Default for EntityCondition {
    fn default() -> Self {
        Self {
            hull: 1.0,
            armor: 1.0,
            readiness: 1.0,
        }
    }
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
