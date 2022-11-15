pub mod data;

use super::*;
use crate::hull::Hull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub struct ShipId(pub u64);

#[derive(Serialize, Deserialize, Clone)]
pub struct Ship {
    pub owner: Option<ClientId>,

    pub ship_data_id: ShipDataId,
    pub rb: RigidBodyHandle,

    pub mobility: Mobility,

    pub hulls: Vec<Option<Hull>>,

    pub wish_pos: Option<na::Vector2<f32>>,
    pub wish_rot: Option<f32>,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Mobility {
    pub linear_acceleration: f32,
    pub angular_acceleration: f32,
    pub max_linear_velocity: f32,
    pub max_angular_velocity: f32,
}
