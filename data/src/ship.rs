use super::*;

pub struct ShipData {
    pub mobility: Mobility,
    /// First is the main hull.
    pub hulls: &'static [BcHullDataId],
    // pub hull_joins: (),
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Mobility {
    pub linear_acceleration: f32,
    pub angular_acceleration: f32,
    pub max_linear_velocity: f32,
    pub max_angular_velocity: f32,
}
