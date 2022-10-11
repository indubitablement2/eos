pub mod defence;
pub mod mobility;

use super::*;
use defence::*;
use mobility::*;

pub struct Hull {
    pub rb: RigidBodyHandle,
    pub defence: Defence,
    pub mobility: Mobility,
}
