// use bevy_ecs::prelude::*;
use gdnative::core_types::TypedArray;
use rapier2d::prelude::*;

/// Modify the game.
pub struct GameParameterRes {
    pub drag: f32, // Velocity is multiplied by this each tick.
}

pub struct TimeRes {
    pub tick: u32,
}

// ! Physic

pub struct PhysicsPipelineRes(pub PhysicsPipeline);
pub struct IntegrationParametersRes(pub IntegrationParameters);
pub struct IslandManagerRes(pub IslandManager);
pub struct BroadPhaseRes(pub BroadPhase);
pub struct NarrowPhaseRes(pub NarrowPhase);
pub struct JointSetRes(pub JointSet);
pub struct CCDSolverRes(pub CCDSolver);
pub struct EventCollectorRes(pub ChannelEventCollector);

pub struct ContactEventReceiverRes(pub crossbeam_channel::Receiver<ContactEvent>);
pub struct IntersectionEventReceiverRes(pub crossbeam_channel::Receiver<IntersectionEvent>);

/// Rigid body set from rapier.
pub struct BodySetRes(pub RigidBodySet);
/// Collider set from rapier.
pub struct ColliderSetRes(pub ColliderSet);

// ! Render

/// All that is needed to render sprites.
pub struct RenderRes {
    pub render_data: Option<TypedArray<f32>>,
    pub visible_instance: i32,
}
