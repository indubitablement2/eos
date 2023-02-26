pub mod builder;
pub mod event_handler;
pub mod userdata;

use super::*;
use event_handler::PhysicsEventCollector;

pub use builder::*;
pub use userdata::*;

const DEFAULT_FRICTION: f32 = 0.6;
const DEFAULT_RESTITUTION: f32 = 0.0;
const DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD: f32 = 0.0;
const DEFAULT_LINEAR_DAMPING: f32 = 0.0;
const DEFAULT_ANGULAR_DAMPING: f32 = 0.0;

pub const GROUP_SHIP: Group = Group::GROUP_1;
pub const GROUP_SHIELD: Group = Group::GROUP_2;
pub const GROUP_DEBRIS: Group = Group::GROUP_3;
pub const GROUP_MISSILE: Group = Group::GROUP_4;
pub const GROUP_FIGHTER: Group = Group::GROUP_5;
pub const GROUP_PROJECTILE: Group = Group::GROUP_6;
pub const GROUP_ALL: Group = GROUP_SHIP
    .union(GROUP_SHIELD)
    .union(GROUP_DEBRIS)
    .union(GROUP_MISSILE)
    .union(GROUP_FIGHTER)
    .union(GROUP_PROJECTILE);

/// Colliders ignore all collider with the same `GroupIgnore`.
pub type GroupIgnore = u32;

#[derive(Serialize, Deserialize)]
pub struct Physics {
    query_pipeline: QueryPipeline,
    #[serde(skip)]
    physics_pipeline: PhysicsPipeline,
    #[serde(skip)]
    #[serde(default = "default_integration_parameters")]
    integration_parameters: IntegrationParameters,
    islands: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    #[serde(skip)]
    events: PhysicsEventCollector,
    next_group_ignore: GroupIgnore,
    /// All entity use this rigid body.
    #[serde(skip)]
    #[serde(default = "default_rb")]
    default_rb: RigidBody,
}
impl Physics {
    pub fn step(&mut self) {
        self.events.clear();

        self.physics_pipeline.step(
            &vector![0.0, 0.0],
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &Hooks,
            &self.events,
        );
    }

    pub fn add_entity(
        &mut self,
        entity_id: EntityId,
        mut collider: Collider,
        copy_group_ignore: Option<RigidBodyHandle>,
        position: na::Isometry2<f32>,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let group_ignore =
            if let Some(rb) = copy_group_ignore.and_then(|handle| self.bodies.get(handle)) {
                rb.user_data.group_ignore()
            } else {
                self.new_group_ignore()
            };

        let user_data = UserData::pack(GenericId::Entity(entity_id), group_ignore);

        let mut rb = self.default_rb.clone();
        rb.user_data = user_data;
        rb.set_position(position, false);
        let rb = self.bodies.insert(rb);

        collider.user_data = user_data;
        let coll = self
            .colliders
            .insert_with_parent(collider, rb, &mut self.bodies);

        (rb, coll)
    }

    // TODO: Enable shield collider on the body.
    pub fn enable_shield(&mut self, enabled: bool) {
        todo!()
    }

    /// Remove the body and its colliders.
    pub fn remove_body(&mut self, handle: RigidBodyHandle) -> RigidBody {
        self.bodies
            .remove(
                handle,
                &mut self.islands,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            )
            .unwrap()
    }

    pub fn intersection_with_shape(
        &self,
        shape_pos: &Isometry<Real>,
        shape: &dyn Shape,
        filter: QueryFilter,
    ) -> Option<ColliderHandle> {
        self.query_pipeline.intersection_with_shape(
            &self.bodies,
            &self.colliders,
            shape_pos,
            shape,
            filter,
        )
    }

    pub fn body(&self, rb: RigidBodyHandle) -> &RigidBody {
        &self.bodies[rb]
    }

    pub fn body_mut(&mut self, rb: RigidBodyHandle) -> &mut RigidBody {
        &mut self.bodies[rb]
    }

    fn new_group_ignore(&mut self) -> GroupIgnore {
        let group_ignore = self.next_group_ignore;
        self.next_group_ignore += 1;
        group_ignore
    }
}

struct Hooks;
impl PhysicsHooks for Hooks {
    fn filter_contact_pair(&self, context: &PairFilterContext) -> Option<SolverFlags> {
        let a = context.colliders[context.collider1]
            .user_data
            .group_ignore();
        let b = context.colliders[context.collider2]
            .user_data
            .group_ignore();
        if a != b {
            Some(SolverFlags::COMPUTE_IMPULSES)
        } else {
            None
        }
    }

    fn filter_intersection_pair(&self, context: &PairFilterContext) -> bool {
        self.filter_contact_pair(context).is_some()
    }

    fn modify_solver_contacts(&self, _context: &mut ContactModificationContext) {}
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            query_pipeline: Default::default(),
            physics_pipeline: Default::default(),
            integration_parameters: default_integration_parameters(),
            islands: Default::default(),
            broad_phase: Default::default(),
            narrow_phase: Default::default(),
            bodies: Default::default(),
            colliders: Default::default(),
            impulse_joints: Default::default(),
            multibody_joints: Default::default(),
            ccd_solver: Default::default(),
            events: Default::default(),
            next_group_ignore: Default::default(),
            default_rb: default_rb(),
        }
    }
}

fn default_integration_parameters() -> IntegrationParameters {
    IntegrationParameters {
        dt: DT,
        min_ccd_dt: DT / 100.0,
        ..Default::default()
    }
}

fn default_rb() -> RigidBody {
    RigidBodyBuilder::dynamic()
        .linear_damping(DEFAULT_LINEAR_DAMPING)
        .angular_damping(DEFAULT_ANGULAR_DAMPING)
        .can_sleep(false)
        .build()
}
