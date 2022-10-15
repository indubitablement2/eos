pub mod defence;

use super::*;
use defence::*;

#[derive(Serialize, Deserialize)]
pub struct Hull {
    pub rb: RigidBodyHandle,
    pub defence: Defence,
}
