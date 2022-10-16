pub mod defence;

use super::*;

pub use defence::*;

#[derive(Serialize, Deserialize)]
pub struct Hull {
    pub hull_data_index: usize,
    pub rb: RigidBodyHandle,
    pub defence: Defence,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HullData {
    pub defence: Defence,
    pub shape: HullShape,
    pub density: f32,
}
