use rapier2d::crossbeam::channel::*;
use rapier2d::prelude::*;

pub struct BodySetBundle {
    pub collider_set: ColliderSet,
    /// Unused.
    pub _rigid_body_set: RigidBodySet,
}
impl BodySetBundle {
    pub fn new() -> Self {
        Self {
            collider_set: ColliderSet::new(),
            _rigid_body_set: RigidBodySet::new(),
        }
    }
}

pub struct CollisionEventsBundle {
    pub contact_recv: Receiver<ContactEvent>,
    pub intersection_recv: Receiver<IntersectionEvent>,
}
impl CollisionEventsBundle {
    pub fn new() -> (Self, ChannelEventCollector) {
        // Initialize the event collector.
        let (contact_send, contact_recv) = unbounded();
        let (intersection_send, intersection_recv) = unbounded();
        let event_handler = ChannelEventCollector::new(intersection_send, contact_send);

        (
            Self {
                contact_recv,
                intersection_recv,
            },
            event_handler,
        )
    }
}

pub struct CollisionPipelineBundle {
    pub collision_pipeline: CollisionPipeline,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub channel_event_collector: ChannelEventCollector,
}
impl CollisionPipelineBundle {
    pub fn new(channel_event_collector: ChannelEventCollector) -> Self {
        Self {
            collision_pipeline: CollisionPipeline::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            channel_event_collector,
        }
    }

    pub fn step(&mut self, body_set_bundle: &mut BodySetBundle) {
        self.collision_pipeline.step(
            0.0,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut body_set_bundle._rigid_body_set,
            &mut body_set_bundle.collider_set,
            &(),
            &self.channel_event_collector,
        );
    }
}

pub struct QueryPipelineBundle {
    pub query_pipeline: QueryPipeline,
    /// Unused.
    pub _island_manager: IslandManager,
}
impl QueryPipelineBundle {
    pub fn new() -> Self {
        Self {
            query_pipeline: QueryPipeline::new(),
            _island_manager: IslandManager::new(),
        }
    }

    /// Update the acceleration structure on the query pipeline.
    pub fn update(&mut self, body_set_bundle: &BodySetBundle) {
        self.query_pipeline.update_with_mode(
            &self._island_manager,
            &body_set_bundle._rigid_body_set,
            &body_set_bundle.collider_set,
            QueryPipelineMode::CurrentPosition,
        );
    }
}
