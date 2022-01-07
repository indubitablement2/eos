use bevy_ecs::prelude::*;
use common::{idx::*, packets::Packet};
use crossbeam::queue::SegQueue;

/// Register the events.
pub fn add_event_res(world: &mut World) {
    world.insert_resource(EventRes::<ClientDisconnected>::new());
    world.insert_resource(EventRes::<FleetIdle>::new());
}

/// A client just disconnected.
pub struct ClientDisconnected {
    /// The client that just disconnected.
    pub client_id: ClientId,
    /// This can be used to try to send a packet to the client before dropping the connection.
    pub send_packet: Option<Packet>,
}

/// A fleet has been without velocity for some time, but does not have an orbit.
pub struct FleetIdle {
    pub entity: Entity
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
