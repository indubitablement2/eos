use super::*;
use std::cell::RefCell;

#[derive(Default)]
pub struct PhysicsEventCollector {
    // collision_event: RefCell<Vec<CollisionEvent>>,
    contact_force_event: RefCell<Vec<ContactForceEvent>>,
}
// We do not use multi-threading.
unsafe impl Sync for PhysicsEventCollector {}
impl PhysicsEventCollector {
    pub fn clear(&mut self) {
        // self.collision_event.borrow_mut().clear();
        self.contact_force_event.borrow_mut().clear();
    }
}
impl EventHandler for PhysicsEventCollector {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        // self.collision_event.borrow_mut().push(event);
    }

    fn handle_contact_force_event(
        &self,
        dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        contact_pair: &ContactPair,
        total_force_magnitude: Real,
    ) {
        let event = ContactForceEvent::from_contact_pair(dt, contact_pair, total_force_magnitude);
        self.contact_force_event.borrow_mut().push(event);
    }
}
