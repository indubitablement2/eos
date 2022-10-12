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
impl Hull {
    pub fn asd(self, bc: &mut Battlescape) {
        // bc.physics.add_body(pos, linvel, angvel, shape, density, memberships, filter, active_events, active_hooks)
    }
}
