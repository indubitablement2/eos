use super::*;
use parking_lot::Mutex;
use std::sync::Arc;

const DEFAULT_LINEAR_DAMPING: f32 = 0.01;
const DEFAULT_ANGULAR_DAMPING: f32 = 0.01;
const DEFAULT_FRICTION: f32 = 0.3;
const DEFAULT_RESTITUTION: f32 = 0.2;
const DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD: f32 = 0.0;

const INTEGRATION_PARAMETERS: IntegrationParameters = IntegrationParameters {
    dt: DT,
    min_ccd_dt: DT / 100.0,
    erp: 0.8,
    damping_ratio: 0.25,
    joint_erp: 1.0,
    joint_damping_ratio: 0.25,
    allowed_linear_error: 0.001,
    max_penetration_correction: f32::MAX,
    prediction_distance: 0.002,
    max_velocity_iterations: 4,
    max_velocity_friction_iterations: 8,
    max_stabilization_iterations: 1,
    interleave_restitution_and_friction_resolution: true,
    min_island_size: 128,
    max_ccd_substeps: 1,
};

pub mod group {
    use super::*;

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

    pub const GROUPS_SHIP: InteractionGroups = InteractionGroups::new(GROUP_SHIP, GROUP_ALL);
}

#[derive(Default)]
pub struct Physics {
    query_pipeline: QueryPipeline,
    physics_pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    pub events: PhysicsEventCollector,
    next_group_ignore: u64,
}
impl Physics {
    pub fn step(&mut self) {
        self.events.0.try_lock().unwrap().clear();

        self.physics_pipeline.step(
            &vector![0.0, 0.0],
            &INTEGRATION_PARAMETERS,
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

    /// group_ignore: Any entity in the same group ignore will not interact.
    /// Can only have one.
    pub fn add_body(
        &mut self,
        position: Isometry2<f32>,
        linvel: Vector2<f32>,
        angvel: f32,

        shape: SharedShape,
        groups: InteractionGroups,
        mprops: MassProperties,

        entity_id: EntityId,
        group_ignore: u64,
    ) -> RigidBodyHandle {
        let rb = RigidBodyBuilder::dynamic()
            .position(position)
            .linvel(linvel)
            .angvel(angvel)
            .user_data(UserData::pack_body(entity_id, group_ignore))
            .linear_damping(DEFAULT_LINEAR_DAMPING)
            .angular_damping(DEFAULT_ANGULAR_DAMPING)
            .build();
        let rb = self.bodies.insert(rb);

        let coll = ColliderBuilder::new(shape)
            .collision_groups(groups)
            .mass_properties(mprops)
            .user_data(UserData::pack_colider(entity_id, false))
            .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS)
            .active_events(ActiveEvents::CONTACT_FORCE_EVENTS)
            .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
            .friction(DEFAULT_FRICTION)
            .restitution(DEFAULT_RESTITUTION)
            .build();

        self.colliders
            .insert_with_parent(coll, rb, &mut self.bodies);

        rb
    }

    // TODO: Add/remove/set shield

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

    // pub fn intersection_with_shape(
    //     &self,
    //     shape_pos: &Isometry<Real>,
    //     shape: &dyn Shape,
    //     filter: QueryFilter,
    // ) -> Option<ColliderHandle> {
    //     self.query_pipeline.intersection_with_shape(
    //         &self.bodies,
    //         &self.colliders,
    //         shape_pos,
    //         shape,
    //         filter,
    //     )
    // }

    pub fn body(&self, rb: RigidBodyHandle) -> &RigidBody {
        &self.bodies[rb]
    }

    pub fn body_mut(&mut self, rb: RigidBodyHandle) -> &mut RigidBody {
        &mut self.bodies[rb]
    }
}

struct Hooks;
impl PhysicsHooks for Hooks {
    fn filter_contact_pair(&self, context: &PairFilterContext) -> Option<SolverFlags> {
        if let Some((rb1, rb2)) = context.rigid_body1.zip(context.rigid_body2) {
            if context.bodies[rb1].user_data.group_ignore()
                == context.bodies[rb2].user_data.group_ignore()
            {
                return None;
            }
        }
        Some(SolverFlags::COMPUTE_IMPULSES)
    }

    fn filter_intersection_pair(&self, context: &PairFilterContext) -> bool {
        self.filter_contact_pair(context).is_some()
    }

    fn modify_solver_contacts(&self, _context: &mut ContactModificationContext) {}
}

#[derive(Debug, Clone, Copy)]
pub struct ContactEvent {
    // pub entity_id: EntityId,
    pub shield: bool,

    pub with_entity_id: EntityId,
    pub with_shield: bool,

    /// The world-space point of the force with strongest magnitude.
    pub point: Point2<f32>,

    /// The world-space (unit) direction of the force with strongest magnitude.
    pub force_direction: Vector2<f32>,
    /// The magnitude of the largest force at a contact point of this contact pair.
    pub force_magnitude: f32,
}

#[derive(Default)]
pub struct PhysicsEventCollector(pub Arc<Mutex<Vec<(EntityId, ContactEvent)>>>);
impl EventHandler for PhysicsEventCollector {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        _event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
    }

    fn handle_contact_force_event(
        &self,
        dt: Real,
        _bodies: &RigidBodySet,
        colliders: &ColliderSet,
        contact_pair: &ContactPair,
        total_force_magnitude: Real,
    ) {
        // TODO: only take event when force at a point is > some value
        // contact_pair.
        let mut point1 = Point2::default();
        log::debug!("num contact manifold: {}", contact_pair.manifolds.len());
        for m in contact_pair.manifolds.iter() {
            // m.points
            log::debug!("contact points: {:?}", m.points);
            for p in m.points.iter() {
                point1 = p.local_p1;
            }
        }

        let event = ContactForceEvent::from_contact_pair(dt, contact_pair, total_force_magnitude);

        let a = colliders[contact_pair.collider1].user_data;
        let b = colliders[contact_pair.collider2].user_data;

        let entity_id = a.entity_id();
        let event = ContactEvent {
            shield: false,

            with_entity_id: b.entity_id(),
            with_shield: false,

            point: point1,
            force_direction: event.max_force_direction,
            force_magnitude: event.max_force_magnitude,
        };

        self.0.try_lock().unwrap().push((entity_id, event));
    }
}

/// Body:
/// - EntityId: 64
/// - Group ignore: 64
/// Collider:
/// - EntityId: 64
/// - Shield: 1
pub trait UserData {
    const ID_TYPE_OFFSET: u32 = u64::BITS;
    const GROUP_IGNORE_OFFSET: u32 = Self::ID_TYPE_OFFSET + 4;
    fn pack_body(entity_id: EntityId, group_ignore: u64) -> Self;
    fn pack_colider(entity_id: EntityId, shield: bool) -> Self;
    fn entity_id(self) -> EntityId;
    /// Only valid for body.
    fn group_ignore(self) -> u64;
    /// Only valid for collider.
    fn shield(self) -> bool;
}
impl UserData for u128 {
    fn pack_body(entity_id: EntityId, group_ignore: u64) -> Self {
        entity_id.as_u64() as u128 | (group_ignore as u128) << 64
    }

    fn pack_colider(entity_id: EntityId, shield: bool) -> Self {
        entity_id.as_u64() as u128 | (shield as u128) << 64
    }

    fn entity_id(self) -> EntityId {
        EntityId::from_u64(self as u64).unwrap()
    }

    fn group_ignore(self) -> u64 {
        (self >> 64) as u64
    }

    fn shield(self) -> bool {
        self >> 64 != 0
    }
}
