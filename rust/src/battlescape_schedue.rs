use crate::battlescape_components::*;
use crate::battlescape_resources::*;
use crate::battlescape_systems::*;
use bevy_ecs::prelude::*;
use gdnative::prelude::*;
use rapier2d::na;
use rapier2d::prelude::*;

pub struct EcsWorld {
    pub world: World,
    pub schedule: Schedule,
}

impl EcsWorld {
    /// Init the ecs by creating a new world or loading one.
    pub fn new() -> EcsWorld {
        EcsWorld {
            world: init_world(),
            schedule: init_schedule(),
        }
    }

    /// Run game logic for one tick.
    pub fn run(&mut self) {
        self.schedule.run_once(&mut self.world);
    }

    /// This just add a ball for now.
    pub fn add_ship(&mut self, position: Vector2, rotation: f32) {
        let mut body_set = unsafe { self.world.get_resource_unchecked_mut::<BodySetRes>().unwrap() };
        let mut collider_set = unsafe { self.world.get_resource_unchecked_mut::<ColliderSetRes>().unwrap() };

        let collider = ColliderBuilder::ball(1.0).build();
        let body = RigidBodyBuilder::new_dynamic()
            .position(na::Isometry2::new(na::vector![position.x, position.y], rotation))
            .build();

        let body_handle = body_set.0.insert(body);
        let collider_handle = collider_set.0.insert_with_parent(collider, body_handle, &mut body_set.0);

        self.world
            .spawn()
            .insert(Renderable {})
            .insert(PhysicBodyHandle(body_handle))
            .insert(PhysicCollisionHandle(collider_handle));
    }
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

    // time

    // move

    // make grid

    // physic

    schedule
}
