use super::*;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct PhysicsEventCollector {
    collision_event: Arc<Mutex<Vec<CollisionEvent>>>,
    contact_force_event: Arc<Mutex<Vec<ContactForceEvent>>>,
}
impl PhysicsEventCollector {
    pub fn clear(&mut self) {
        self.collision_event.try_lock().unwrap().clear();
        self.contact_force_event.try_lock().unwrap().clear();
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
        self.collision_event.try_lock().unwrap().push(event);
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
        self.contact_force_event.try_lock().unwrap().push(event);
    }
}
