use crate::ecs_components::ClientId;
use bevy_ecs::prelude::*;
use crossbeam_queue::SegQueue;

/// Register the events.
pub fn add_event_res(world: &mut World) {
    world.insert_resource(EventRes::<ClientConnected>::new());
    world.insert_resource(EventRes::<ClientDisconnected>::new());
    world.insert_resource(EventRes::<JustControlled>::new());
    world.insert_resource(EventRes::<JustStopControlled>::new());
}

/// A client just connected.
pub struct ClientConnected {
    /// The client that just connected.
    pub client_id: ClientId,
}

/// A client just disconnected.
pub struct ClientDisconnected {
    /// The client that just disconnected.
    pub client_id: ClientId,
}

/// This entity has just started being controlled.
pub struct JustControlled {
    /// The entity that is now being controlled.
    pub entity: Entity,
    /// The client that now control the entity.
    pub client_id: ClientId,
}

/// This entity has just stopped being controlled.
pub struct JustStopControlled {
    /// The entity that was being controlled.
    pub entity: Entity,
    /// The client that was controlling the entity.
    pub client_id: ClientId,
}

pub struct EventRes<T> {
    /// Contain events triggered by preceding systems.
    pub events: SegQueue<T>,
}
impl<T> EventRes<T> {
    fn new() -> Self {
        Self {
            events: SegQueue::new(),
        }
    }
}
