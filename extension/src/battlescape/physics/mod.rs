pub mod builder;
pub mod event_handler;
pub mod userdata;

use super::*;
use event_handler::PhysicsEventCollector;

pub use builder::*;
pub use userdata::*;

/// Colliders ignore all collider with the same `GroupIgnore`.
pub type GroupIgnore = u64;

#[derive(Serialize, Deserialize, Default)]
pub struct Physics {
    query_pipeline: QueryPipeline,
    #[serde(skip)]
    physics_pipeline: PhysicsPipeline,
    #[serde(skip)]
    integration_parameters: CustomIntegrationParameters,
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
}
impl Physics {
    pub fn step(&mut self) {
        self.events.clear();

        self.physics_pipeline.step(
            &vector![0.0, 0.0],
            &self.integration_parameters.0,
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

    pub fn add_body(
        &mut self,
        builder: SimpleRigidBodyBuilder,
        id: BodyGenericId,
    ) -> RigidBodyHandle {
        let group_ignore = if let Some(rb) = builder
            .copy_group_ignore
            .and_then(|handle| self.bodies.get(handle))
        {
            rb.user_data.group_ignore()
        } else {
            self.new_group_ignore()
        };

        let rb = builder
            .builder
            .user_data(UserData::pack_body(id, group_ignore))
            .build();

        self.bodies.insert(rb)
    }

    pub fn add_collider(
        &mut self,
        builder: SimpleColliderBuilder,
        parent_handle: RigidBodyHandle,
        id: ColliderGenericId,
    ) -> ColliderHandle {
        let group_ignore = if let Some(rb) = self.bodies.get(parent_handle) {
            rb.user_data.group_ignore()
        } else {
            self.new_group_ignore()
        };

        let coll = builder
            .builder
            .user_data(UserData::pack_collider(id, group_ignore))
            .build();

        self.colliders
            .insert_with_parent(coll, parent_handle, &mut self.bodies)
    }

    /// Remove the body and its colliders.
    /// ## Panic:
    /// Handle is invalid.
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

    /// ## Panic:
    /// Handle is invalid.
    pub fn remove_collider(&mut self, handle: ColliderHandle) -> Collider {
        self.colliders
            .remove(handle, &mut self.islands, &mut self.bodies, false)
            .unwrap()
    }

    pub fn intersection_any_with_shape(
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

struct CustomIntegrationParameters(IntegrationParameters);
impl Default for CustomIntegrationParameters {
    fn default() -> Self {
        Self(IntegrationParameters {
            dt: DT,
            min_ccd_dt: DT / 100.0,
            ..Default::default()
        })
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
