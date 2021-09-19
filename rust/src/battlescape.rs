use std::time::Instant;

use crate::ecs_components::*;
use crate::ecs_resources::*;
use crate::ecs_systems::*;
use bevy_ecs::prelude::*;
use gdnative::prelude::*;
use rapier2d::na;
use rapier2d::prelude::*;
use crossbeam_channel::*;

pub struct UpdateRequest {
    /// How many times the ecs should update before returning a result.
    pub num_update: u32,
    pub send_render: bool,
    pub spawn_ship: Option<(Vector2, f32)>,
}

pub struct UpdateResult {
    /// How many update were completed.
    pub num_update: u32,
}

pub struct Battlescape {
    /// How many updates are in queue.
    pub pending_update: u32,
    request_sender: Sender<UpdateRequest>,
    result_receiver: Receiver<UpdateResult>,
}

impl Battlescape {
    /// Init the ecs by creating a new world or loading one and a default schedule.
    pub fn new() -> Battlescape {
        let (request_sender, request_receiver) = unbounded();
        let (result_sender, result_receiver) = unbounded();

        std::thread::spawn(move || {
            battlescape_runner(request_receiver, result_sender)
        });

        Battlescape {
            pending_update: 0,
            request_sender,
            result_receiver,
        }
    }

    /// Send an update request to the ecs.
    pub fn update(&mut self, mut update_request: UpdateRequest) -> Result<(), crossbeam_channel::SendError<UpdateRequest>> {
        update_request.num_update = update_request.num_update.max(1);
        self.pending_update += update_request.num_update;
        self.request_sender.send(update_request)
    }

    /// Wait at most untill deadline for an update from the ecs.
    pub fn wait_to_complete(&mut self, deadline: Instant) -> Result<UpdateResult, crossbeam_channel::RecvTimeoutError> {
        let result = self.result_receiver.recv_deadline(deadline);

        if let Ok(update_result) = &result {
            self.pending_update -= update_result.num_update;
        }

        result
    }
}

fn battlescape_runner(request_receiver: Receiver<UpdateRequest>, result_sender: Sender<UpdateResult>) {
    let mut world = init_world();
    let mut schedule = init_schedule();

    while let Ok(update_request) = request_receiver.recv() {
        // Spawn a ship.
        if let Some((pos, rot)) = update_request.spawn_ship {
            add_ship(&mut world, pos, rot);
        }

        schedule.run_once(&mut world);

        // Send result back.
        if result_sender.send(UpdateResult {
            num_update: 1,
        }).is_err() {
            break;
        }
    }
}

/// This just add a ball for now.
fn add_ship(world: &mut World, position: Vector2, rotation: f32) {
    let mut body_set = unsafe { world.get_resource_unchecked_mut::<BodySetRes>().unwrap() };
    let mut collider_set = unsafe { world.get_resource_unchecked_mut::<ColliderSetRes>().unwrap() };

    let collider = ColliderBuilder::ball(1.0).build();
    let body = RigidBodyBuilder::new_dynamic()
        .position(na::Isometry2::new(na::vector![position.x, position.y], rotation))
        .build();

    let body_handle = body_set.0.insert(body);
    let collider_handle = collider_set.0.insert_with_parent(collider, body_handle, &mut body_set.0);

    world.spawn()
        .insert(Renderable {})
        .insert(PhysicBodyHandle(body_handle))
        .insert(PhysicCollisionHandle(collider_handle));
}

fn init_world() -> World {
    let mut world = World::default();

    // Create and insert physic resources.
    world.insert_resource(PhysicsPipelineRes(PhysicsPipeline::new()));
    world.insert_resource(IntegrationParametersRes(IntegrationParameters::default()));
    world.insert_resource(IslandManagerRes(IslandManager::new()));
    world.insert_resource(BroadPhaseRes(BroadPhase::new()));
    world.insert_resource(NarrowPhaseRes(NarrowPhase::new()));
    world.insert_resource(JointSetRes(JointSet::new()));
    world.insert_resource(CCDSolverRes(CCDSolver::new()));
    world.insert_resource(BodySetRes(RigidBodySet::new()));
    world.insert_resource(ColliderSetRes(ColliderSet::new()));
    let (intersection_sender, intersection_receiver) = crossbeam_channel::unbounded();
    let (contact_sender, contact_receiver) = crossbeam_channel::unbounded();
    world.insert_resource(EventCollectorRes(ChannelEventCollector::new(
        intersection_sender,
        contact_sender,
    )));
    world.insert_resource(IntersectionEventReceiverRes(intersection_receiver));
    world.insert_resource(ContactEventReceiverRes(contact_receiver));

    // Insert other resources.
    world.insert_resource(TimeRes { tick: 0 });
    world.insert_resource(GameParameterRes { drag: 0.75 });

    world
}

/// Create a new default schedule.
fn init_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    // Insert stages.
    schedule.add_stage("pre_update", SystemStage::parallel());
    schedule.add_system_to_stage("pre_update", time_system.system());

    schedule.add_stage_after("pre_update", "update", SystemStage::parallel());
    schedule.add_system_to_stage("update", physic_system.system());

    schedule.add_stage_after("update", "post_update", SystemStage::single_threaded());

    schedule
}
