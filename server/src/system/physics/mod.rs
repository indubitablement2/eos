pub mod builder;
pub mod event_handler;
pub mod user_data;

use self::user_data::*;
use super::*;
pub use builder::*;
use event_handler::PhysicsEventCollector;

pub type PhysicsTeam = u16;

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
    team_id_dispenser: SmallIdDispenser,
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
            &mut Hooks,
            &self.events,
        );
    }

    /// `ignore_group` is used by missile/projectile type
    /// to avoid hitting their parrent as they spawn.
    pub fn add_entity(
        &mut self,
        id: LocalEntityId,
        team: PhysicsTeam,
        ignore_group: Option<LocalEntityId>,
        entity_data: &EntityData,
        position: Isometry2<f32>,
        linvel: Vector2<f32>,
        angvel: f32,
    ) -> (RigidBodyHandle, Option<ColliderHandle>) {
        let ignore_group = if let Some(ignore_group) = ignore_group {
            ignore_group
        } else {
            id
        };

        let user_data = UserData {
            id,
            ignore_group,
            team,
            wish_ignore_same_team: entity_data.wish_ignore_same_team,
            force_ignore_same_team: entity_data.force_ignore_same_team,
        }
        .pack() as u128;

        let mut body = entity_data.body.clone();
        body.user_data = user_data;
        body.set_position(position, false);
        body.set_linvel(linvel, false);
        body.set_angvel(angvel, false);

        let handle = self.bodies.insert(body);

        let collider_handles = entity_data.hull.as_ref().map(|hull_data| {
            let mut collider = hull_data.collider.clone();

            collider.user_data = user_data;

            self.colliders
                .insert_with_parent(collider, handle, &mut self.bodies)
        });

        (handle, collider_handles)
    }

    /// Remove the body and its colliders.
    pub fn remove_body(&mut self, handle: RigidBodyHandle) {
        let _ = self.bodies.remove(
            handle,
            &mut self.islands,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
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
        self.filter_intersection_pair(context)
            .then_some(SolverFlags::COMPUTE_IMPULSES)
    }

    fn filter_intersection_pair(&self, context: &PairFilterContext) -> bool {
        let a = UserData::unpack(context.colliders[context.collider1].user_data as u64);
        let b = UserData::unpack(context.colliders[context.collider2].user_data as u64);

        a.test(b)
    }

    fn modify_solver_contacts(&self, _context: &mut ContactModificationContext) {}
}
