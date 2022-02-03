use bevy_ecs::prelude::*;
use common::{idx::*, net::packets::Packet};
use crossbeam::queue::SegQueue;

/// Register the events.
pub fn add_event_res(world: &mut World) {
    world.insert_resource(EventRes::<ClientDisconnected>::new());
    world.insert_resource(EventRes::<FleetIdle>::new());
    world.insert_resource(EventRes::<FleetDestroyed>::new());
}

/// A client just disconnected.
pub struct ClientDisconnected {
    /// The client that just disconnected.
    pub client_id: ClientId,
    pub fleet_entity: Entity,
    /// This can be used to try to send a packet to the client before dropping the connection.
    pub send_packet: Option<Packet>,
}

/// A fleet has been without velocity for some time and does not have an orbit.
pub struct FleetIdle {
    pub entity: Entity,
}

/// All ships from a fleet have been removed.
/// This could be a client or an ai fleet.
pub struct FleetDestroyed {
    pub entity: Entity,
}

/// Contain events triggered by preceding systems.
pub struct EventRes<T> {
    events: SegQueue<T>,
}
impl<T> EventRes<T> {
    fn new() -> Self {
        Self {
            events: SegQueue::new(),
        }
    }

    pub fn push(&self, event: T) {
        self.events.push(event);
    }

    pub fn pop(&self) -> Option<T> {
        self.events.pop()
    }
}
