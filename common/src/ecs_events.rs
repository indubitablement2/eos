use bevy_ecs::prelude::*;

/// Register the events.
pub fn add_event_handlers(world: &mut World) {
    world.insert_resource(EventRes::<TestEvent>::new());
}
pub fn clear_events(mut test_event: ResMut<EventRes<TestEvent>>) {
    test_event.clear_events();
}

pub struct TestEvent {
    pub entity: Entity,
    data2: f32,
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

