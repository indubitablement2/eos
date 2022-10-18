pub mod group;
pub mod shape;
mod user_data;

use self::user_data::UserData;
use super::*;
use std::sync::{Arc, Mutex};

pub use self::shape::*;
pub use group::*;

const DEFAULT_BODY_FRICTION: f32 = 0.3;
const DEFAULT_BODY_RESTITUTION: f32 = 0.2;
const DEFAULT_FORCE_EVENT_THRESHOLD: f32 = 1.0;

#[derive(Serialize, Deserialize)]
pub struct Physics {
    pub query_pipeline: QueryPipeline,
    #[serde(skip)]
    physics_pipeline: PhysicsPipeline,
    integration_parameters: IntegrationParameters,
    islands: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    #[serde(skip)]
    pub events: PhysicsEventCollector,
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
            &user_data::Hooks,
            &self.events,
        );

        self.query_pipeline
            .update(&self.islands, &self.bodies, &self.colliders);
    }

    pub fn add_body(
        &mut self,
        pos: na::Isometry2<f32>,
        linvel: na::Vector2<f32>,
        angvel: f32,
        shape: SharedShape,
        density: f32,
        ignore_rb: Option<RigidBodyHandle>,
        team: Option<u32>,
        team_ignore: Option<u32>,
        memberships: PhysicsGroup,
        filter: PhysicsGroup,
    ) -> RigidBodyHandle {
        let rb = RigidBodyBuilder::dynamic()
            .position(pos)
            .linvel(linvel)
            .angvel(angvel)
            .build();
        let rb_handle = self.bodies.insert(rb);

        let ignore_rb = if let Some(ignore_rb) = ignore_rb {
            // Copy the rb handle from another rb.
            if let Some(coll) = self
                .bodies
                .get(ignore_rb)
                .and_then(|body| body.colliders().first())
                .and_then(|c_handle| self.colliders.get(*c_handle))
            {
                UserData::get_rb_ignore(coll.user_data)
            } else {
                // Something failed. Use our rb handle instead.
                rb_handle
            }
        } else {
            // Use our rb handle so that other can ignore us.
            rb_handle
        };
        let user_data = UserData::build(Some(ignore_rb), team_ignore, team);

        let coll = ColliderBuilder::new(shape)
            .density(density)
            .friction(DEFAULT_BODY_FRICTION)
            .restitution(DEFAULT_BODY_RESTITUTION)
            .collision_groups(InteractionGroups::new(memberships.into(), filter.into()))
            .active_events(ActiveEvents::all())
            .contact_force_event_threshold(DEFAULT_FORCE_EVENT_THRESHOLD)
            .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::FILTER_INTERSECTION_PAIR)
            .user_data(user_data)
            .build();
        self.colliders
            .insert_with_parent(coll, rb_handle, &mut self.bodies);

        rb_handle
    }
}
impl Default for Physics {
    fn default() -> Self {
        let integration_parameters = IntegrationParameters {
            dt: Battlescape::TICK_DURATION_SEC,
            min_ccd_dt: Battlescape::TICK_DURATION_SEC / 100.0,
            ..Default::default()
        };
        Self {
            query_pipeline: Default::default(),
            physics_pipeline: Default::default(),
            integration_parameters,
            islands: Default::default(),
            broad_phase: Default::default(),
            narrow_phase: Default::default(),
            bodies: Default::default(),
            colliders: Default::default(),
            impulse_joints: Default::default(),
            multibody_joints: Default::default(),
            ccd_solver: Default::default(),
            events: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct PhysicsEventCollector {
    collision_event: Arc<Mutex<Vec<CollisionEvent>>>,
    contact_force_event: Arc<Mutex<Vec<ContactForceEvent>>>,
}
impl PhysicsEventCollector {
    pub fn clear(&mut self) {
        self.collision_event.try_lock().unwrap().clear();
        self.contact_force_event.try_lock().unwrap().clear();
    }
}
impl EventHandler for PhysicsEventCollector {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        self.collision_event.try_lock().unwrap().push(event);
    }

    fn handle_contact_force_event(
        &self,
        dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        contact_pair: &ContactPair,
        total_force_magnitude: Real,
    ) {
        let event = ContactForceEvent::from_contact_pair(dt, contact_pair, total_force_magnitude);
        self.contact_force_event.try_lock().unwrap().push(event);
    }
}
