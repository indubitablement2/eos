pub mod defence;

use super::*;
use defence::*;

#[derive(Serialize, Deserialize)]
pub struct Hull {
    pub rb: RigidBodyHandle,
    pub defence: Defence,
}
impl Hull {
    pub fn asd(self, bc: &mut Battlescape) {
        // bc.physics.add_body(pos, linvel, angvel, shape, density, memberships, filter, active_events, active_hooks)
    }
}
