pub mod user_data;

use super::*;
use std::sync::{Arc, Mutex};

pub use self::shape::*;

pub const DEFAULT_BODY_FRICTION: f32 = 0.3;
pub const DEFAULT_BODY_RESTITUTION: f32 = 0.2;
pub const DEFAULT_FORCE_EVENT_THRESHOLD: f32 = 1.0;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericId {
    ShipId(ShipId),
    HullId(HullId),
}
impl GenericId {
    pub fn from_ship_id(ship_id: ShipId) -> Self {
        Self::ShipId(ship_id)
    }

    pub fn from_hull_id(hull_id: HullId) -> Self {
        Self::HullId(hull_id)
    }

    /// Return (id_type, id)
    pub fn pack(self) -> (u8, u32) {
        match self {
            GenericId::ShipId(id) => (0, id.0),
            GenericId::HullId(id) => (1, id.0),
        }
    }

    pub fn unpack(id_type: u8, id: u32) -> Self {
        match id_type {
            0 => Self::ShipId(ShipId(id)),
            1 => Self::HullId(HullId(id)),
            _ => unreachable!(),
        }
    }
}

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
    next_group_ignore: u32,
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

    pub fn new_group_ignore(&mut self) -> u32 {
        let group_ignore = self.next_group_ignore;
        self.next_group_ignore = self.next_group_ignore.wrapping_add(1);
        group_ignore
    }

    pub fn insert_collider(
        &mut self,
        parent_handle: RigidBodyHandle,
        coll: Collider,
    ) -> ColliderHandle {
        self.colliders
            .insert_with_parent(coll, parent_handle, &mut self.bodies)
    }

    // pub fn add_body(
    //     &mut self,
    //     pos: na::Isometry2<f32>,
    //     linvel: na::Vector2<f32>,
    //     angvel: f32,
    //     shape: SharedShape,
    //     density: f32,
    //     groups: InteractionGroups,
    //     dominance_group: i8,
    //     team: u32,
    //     ignore_team: bool,
    //     group_ignore: u32,
    //     hull_id: HullId,
    // ) -> RigidBodyHandle {
    //     let rb = RigidBodyBuilder::dynamic()
    //         .position(pos)
    //         .linvel(linvel)
    //         .angvel(angvel)
    //         .dominance_group(dominance_group)
    //         .build();
    //     let rb_handle = self.bodies.insert(rb);

    //     let user_data = UserData::build(team, group_ignore, hull_id.0, ignore_team);

    //     let coll = ColliderBuilder::new(shape)
    //         .density(density)
    //         .friction(DEFAULT_BODY_FRICTION)
    //         .restitution(DEFAULT_BODY_RESTITUTION)
    //         .collision_groups(groups)
    //         .active_events(ActiveEvents::all())
    //         .contact_force_event_threshold(DEFAULT_FORCE_EVENT_THRESHOLD)
    //         .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::FILTER_INTERSECTION_PAIR)
    //         .user_data(user_data)
    //         .build();
    //     self.colliders
    //         .insert_with_parent(coll, rb_handle, &mut self.bodies);

    //     rb_handle
    // }
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
            next_group_ignore: Default::default(),
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
