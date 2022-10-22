pub mod defence;

use super::*;

pub use defence::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Hull {
    pub hull_data_index: usize,
    pub rb: RigidBodyHandle,
    pub defence: Defence,
}
