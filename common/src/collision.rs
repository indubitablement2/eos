use crossbeam_channel::*;
use rapier2d::prelude::*;

pub struct QueryBundle {
    pub query_pipeline: QueryPipeline,
    pub collider_set: ColliderSet,
    _island_manager: IslandManager,
    _rigid_body_set: RigidBodySet,
}
impl QueryBundle {
    pub fn new() -> Self {
        Self {
            query_pipeline: QueryPipeline::new(),
            collider_set: ColliderSet::new(),
            _island_manager: IslandManager::new(),
            _rigid_body_set: RigidBodySet::new(),
        }
    }

    /// Update the acceleration structure on the query pipeline.
    pub fn update(&mut self) {
        self.query_pipeline.update_with_mode(
            &self._island_manager,
            &self._rigid_body_set,
            &mut self.collider_set,
            QueryPipelineMode::CurrentPosition,
        );
    }

    pub fn remove_collider(&mut self, collider_handle: ColliderHandle) -> Option<Collider> {
        self.collider_set
            .remove(collider_handle, &mut self._island_manager, &mut self._rigid_body_set, false)
    }
}

/// A physics event handler that collects only intersection events into a crossbeam channel.
/// It discard collision events.
struct IntersectionEventsCollector {
    pub intersection_event_sender: Sender<IntersectionEvent>,
}
impl EventHandler for IntersectionEventsCollector {
    fn handle_intersection_event(&self, event: IntersectionEvent) {
        let _ = self.intersection_event_sender.send(event);
    }

    fn handle_contact_event(&self, _: ContactEvent, _: &ContactPair) {}
}

pub struct CollisionPipelineBundle {
    collision_pipeline: CollisionPipeline,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    intersection_events_collector: IntersectionEventsCollector,
}
impl CollisionPipelineBundle {
    pub fn new() -> (Self, Receiver<IntersectionEvent>) {
        // Create the events channel.
        let (intersection_event_sender, intersection_event_receiver) = unbounded();
        (
            Self {
                collision_pipeline: CollisionPipeline::new(),
                broad_phase: BroadPhase::new(),
                narrow_phase: NarrowPhase::new(),
                intersection_events_collector: IntersectionEventsCollector {
                    intersection_event_sender,
                },
            },
            intersection_event_receiver,
        )
    }

    pub fn update(&mut self, collider_set: &mut ColliderSet) {
        self.collision_pipeline.step(
            0.0,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut RigidBodySet::new(),
            collider_set,
            &(),
            &self.intersection_events_collector,
        );
    }
}

pub struct QueryPipelineBundle {
    pub query_pipeline: QueryPipeline,
    /// Unused.
    _island_manager: IslandManager,
    // collider_delete_queue: Vec<ColliderHandle>,
}
impl QueryPipelineBundle {
    pub fn new() -> Self {
        Self {
            query_pipeline: QueryPipeline::new(),
            _island_manager: IslandManager::new(),
        }
    }

    /// Update the acceleration structure on the query pipeline.
    pub fn update(&mut self, collider_set: &ColliderSet) {
        self.query_pipeline.update_with_mode(
            &self._island_manager,
            &RigidBodySet::new(),
            collider_set,
            QueryPipelineMode::CurrentPosition,
        );
    }

    // pub fn delete_collider(&self, collider_set: &mut ColliderSet) {
    //     collider_set.remove(handle, islands, bodies, wake_up)
    // }
}
