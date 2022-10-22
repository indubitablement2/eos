mod mobility;

use super::*;

pub use mobility::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Ship {
    pub ship_data_index: usize,
    pub mobility: Mobility,
    /// First is the main hull.
    pub hulls_index: SmallVec<[Index; 4]>,
}

pub struct ShipBuilder {
    pub ship_data_index: usize,
    pub pos: na::Isometry2<f32>,
    pub linvel: na::Vector2<f32>,
    pub angvel: f32,
    pub team: u32,
}
