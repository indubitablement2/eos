use super::*;
use parking_lot::Mutex;
use std::sync::Arc;

const DEFAULT_LINEAR_DAMPING: f32 = 0.01;
const DEFAULT_ANGULAR_DAMPING: f32 = 0.01;
const DEFAULT_FRICTION: f32 = 0.3;
const DEFAULT_RESTITUTION: f32 = 0.2;
const DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD: f32 = 0.0;

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

#[derive(Serialize, Deserialize, Default)]
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
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    #[serde(skip)]
    events: PhysicsEventCollector,
}
impl Physics {
    pub fn step(&mut self) {
        self.events.0.try_lock().unwrap().clear();

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

    pub fn events(&self) -> &[(EntityId, ContactEvent)] {
        self.events.0.try_lock().unwrap().as_slice()
    }

    /// mprops: Use `MassProperties::from_ball`.
    /// That way center of mass is always at the local origin.
    ///
    /// ignore: Optional EntityId to ignore collisions with.
    /// Useful for ignoring collisions with the entity that spawned the body.
    /// Can only have one.
    pub fn add_body(
        &mut self,
        position: Isometry2<f32>,
        linvel: Vector2<f32>,
        angvel: f32,
        mprops: MassProperties,

        entity_id: EntityId,
        ignore: Option<EntityId>,
    ) -> RigidBodyHandle {
        let rb = RigidBodyBuilder::dynamic()
            .position(position)
            .linvel(linvel)
            .angvel(angvel)
            .additional_mass_properties(mprops)
            .user_data(UserData::pack_body(entity_id, ignore))
            .linear_damping(DEFAULT_LINEAR_DAMPING)
            .angular_damping(DEFAULT_ANGULAR_DAMPING)
            .build();

        self.bodies.insert(rb)
    }

    /// Collider do not affect body's mass properties.
    pub fn add_collider(
        &mut self,
        shape: SharedShape,
        groups: InteractionGroups,
        position: Isometry2<f32>,

        entity_id: EntityId,
        collider_id: ColliderGenericId,

        parent_handle: RigidBodyHandle,
    ) -> ColliderHandle {
        let coll = ColliderBuilder::new(shape)
            .collision_groups(groups)
            .position(position)
            .user_data(UserData::pack_collider(entity_id, collider_id))
            .mass_properties(MassProperties::default())
            .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS)
            .active_events(ActiveEvents::CONTACT_FORCE_EVENTS)
            .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
            .friction(DEFAULT_FRICTION)
            .restitution(DEFAULT_RESTITUTION)
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
}

fn default_integration_parameters() -> IntegrationParameters {
    IntegrationParameters {
        dt: DT,
        min_ccd_dt: DT / 100.0,
        ..Default::default()
    }
}

struct Hooks;
impl PhysicsHooks for Hooks {
    fn filter_contact_pair(&self, context: &PairFilterContext) -> Option<SolverFlags> {
        if let Some((rb1, rb2)) = context.rigid_body1.zip(context.rigid_body2) {
            let a = context.bodies[rb1].user_data;
            let b = context.bodies[rb2].user_data;
            if Some(a.entity_id()) == b.body_entity_ignore()
                || Some(b.entity_id()) == a.body_entity_ignore()
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
    pub collider_id: ColliderGenericId,

    pub with_entity_id: EntityId,
    pub with_collider_id: ColliderGenericId,

    /// The world-space (unit) direction of the force with strongest magnitude.
    pub max_force_direction: Vector2<f32>,
    /// The magnitude of the largest force at a contact point of this contact pair.
    pub max_force_magnitude: f32,
}

#[derive(Default)]
struct PhysicsEventCollector(Arc<Mutex<Vec<(EntityId, ContactEvent)>>>);
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
        let event = ContactForceEvent::from_contact_pair(dt, contact_pair, total_force_magnitude);

        let a = colliders[contact_pair.collider1].user_data;
        let b = colliders[contact_pair.collider2].user_data;

        let entity_id = a.entity_id();
        let event = ContactEvent {
            collider_id: a.collider_collider_id(),
            with_entity_id: b.entity_id(),
            with_collider_id: b.collider_collider_id(),
            max_force_direction: event.max_force_direction,
            max_force_magnitude: event.max_force_magnitude,
        };

        self.0.try_lock().unwrap().push((entity_id, event));
    }
}

/// Possible id of a collider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColliderGenericId {
    HullIdx(u16),
    ShieldIdx(u16),
}
impl ColliderGenericId {
    fn pack(self) -> u32 {
        match self {
            ColliderGenericId::HullIdx(idx) => idx as u32,
            ColliderGenericId::ShieldIdx(idx) => (idx as u32) | (1 << 16),
        }
    }

    fn unpack(data: u32) -> Self {
        let collider_idx = data as u16;
        let collider_type = data >> 16;
        match collider_type {
            0 => ColliderGenericId::HullIdx(collider_idx),
            1 => ColliderGenericId::ShieldIdx(collider_idx),
            _ => unreachable!(),
        }
    }
}

/// Body:
/// - EntityId: 64
/// - EntityId ignore: 64
///
/// Collider:
/// - EntityId: 64
/// - ColliderIdx: 16
/// - ColliderType: 16
/// - unused: 32
pub trait UserData {
    const ID_TYPE_OFFSET: u32 = u64::BITS;
    const GROUP_IGNORE_OFFSET: u32 = Self::ID_TYPE_OFFSET + 4;
    fn pack_body(entity_id: EntityId, ignore: Option<EntityId>) -> Self;
    fn pack_collider(entity_id: EntityId, collider_id: ColliderGenericId) -> Self;
    fn entity_id(self) -> EntityId;
    /// Only valid if this was taken from a body's user data.
    fn body_entity_ignore(self) -> Option<EntityId>;
    /// Only valid if this was taken from a collider's user data.
    fn collider_collider_id(self) -> ColliderGenericId;
}
impl UserData for u128 {
    fn pack_body(entity_id: EntityId, ignore: Option<EntityId>) -> Self {
        entity_id.as_u64() as u128 | (ignore.unwrap_or(entity_id).as_u64() as u128) << 64
    }

    fn pack_collider(entity_id: EntityId, collider_id: ColliderGenericId) -> Self {
        entity_id.as_u64() as u128 | (collider_id.pack() as u128) << 64
    }

    fn entity_id(self) -> EntityId {
        EntityId::from_u64(self as u64).unwrap()
    }

    fn body_entity_ignore(self) -> Option<EntityId> {
        EntityId::from_u64((self >> 64) as u64)
    }

    fn collider_collider_id(self) -> ColliderGenericId {
        let data = (self >> 64) as u32;
        ColliderGenericId::unpack(data)
    }
}
