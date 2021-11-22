use crate::res_clients::ClientId;
use bevy_ecs::prelude::*;

/// Register the events.
pub fn add_event_handlers(world: &mut World) {
    world.insert_resource(EventRes::<ClientConnected>::new());
    world.insert_resource(EventRes::<ClientDisconnected>::new());
    world.insert_resource(EventRes::<JustControlled>::new());
    world.insert_resource(EventRes::<JustStopControlled>::new());
}
pub fn clear_events(
    mut client_connected: ResMut<EventRes<ClientConnected>>,
    mut client_disconnected: ResMut<EventRes<ClientDisconnected>>,
    mut just_controlled: ResMut<EventRes<JustControlled>>,
    mut just_stop_controlled: ResMut<EventRes<JustStopControlled>>,
) {
    client_connected.clear_events();
    client_disconnected.clear_events();
    just_controlled.clear_events();
    just_stop_controlled.clear_events();
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
    event_buffer: Vec<T>,
}
impl<T> EventRes<T> {
    fn new() -> Self {
        Self {
            event_buffer: Vec::new(),
        }
    }

    /// Add an event subsequent systems will be hable to read.
    pub fn trigger_event(&mut self, event: T) {
        self.event_buffer.push(event);
    }

    /// Get events preceding systems have triggered.
    pub fn get_events(&self) -> std::slice::Iter<'_, T> {
        self.event_buffer.iter()
    }

    fn clear_events(&mut self) {
        self.event_buffer.clear();
    }
}
