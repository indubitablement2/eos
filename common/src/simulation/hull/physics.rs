use super::*;
use parking_lot::Mutex;
use std::{num::NonZeroU32, sync::Arc};

const DEFAULT_LINEAR_DAMPING: f32 = 0.01;
const DEFAULT_ANGULAR_DAMPING: f32 = 0.01;
const DEFAULT_FRICTION: f32 = 0.3;
const DEFAULT_RESTITUTION: f32 = 0.2;
const DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD: f32 = 0.0;

const PHYSIC_SCALE: f32 = 1.0 / 64.0;

// TODO: Change this to an enum
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
    // pub const GROUPS_ENTITY: InteractionGroups = InteractionGroups::new(GROUP_SHIP, GROUP_ALL);
}

// TODO: Add query pipeline
#[derive(Default)]
pub struct Hulls {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    next_collision_group_ignore: u64,

    next_hull_id: HullId,
    hulls: IndexMap<HullId, Hull>,
}
impl Hulls {
    pub fn insert(&mut self, save: HullSave) -> (HullId, &mut Hull) {
        let mut hull = Hull::new(save);
        hull.collision_group_ignore = self.next_collision_group_ignore;
        self.next_collision_group_ignore += 1;

        // TODO: Add body and collider.

        let hull_id = self.next_hull_id;
        self.next_hull_id.next();
        let hull = self.hulls.entry(hull_id).or_insert(hull);

        (hull_id, hull)
    }

    pub fn get(&self, hull_id: HullId) -> Option<&Hull> {
        self.hulls.get(&hull_id)
    }

    pub fn get_mut(&mut self, hull_id: HullId) -> Option<&mut Hull> {
        self.hulls.get_mut(&hull_id)
    }

    pub fn iter(&mut self) -> indexmap::map::Iter<'_, hull::HullId, hull::Hull> {
        self.hulls.iter()
    }

    pub fn iter_mut(&mut self) -> indexmap::map::IterMut<'_, hull::HullId, hull::Hull> {
        self.hulls.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.hulls.len()
    }
}

/// Hulls always have 1 rigid body made of 1 collider.
///
/// Bodies are always a hull.
/// Colliders are either a hull or a shield.
#[derive(Default)]
pub struct Physics {
    pub hulls: Hulls,

    // TODO: Streamline this for:
    // - projectile (query point)
    // - beam(raycast)
    // - turret/hull (query broadphase)
    query_pipeline: QueryPipeline,
    physics_pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    events: PhysicsEventCollector,
}
impl Physics {
    pub fn step(&mut self) {
        // Sync from hulls.
        for hull in self.hulls.hulls.values() {
            let body = &mut self.hulls.bodies[hull.rb];
            body.set_position(
                Isometry2::from_parts(
                    na::Translation {
                        vector: hull.position * PHYSIC_SCALE,
                    },
                    hull.rotation,
                ),
                true,
            );
            body.set_linvel(hull.linvel * PHYSIC_SCALE, true);
            body.set_angvel(hull.angvel, true);
            body.user_data.set_group_ignore(hull.collision_group_ignore);
        }

        self.events.0.try_lock().unwrap().clear();

        let integration_parameters = IntegrationParameters {
            dt: DT.as_secs_f32(),
            min_ccd_dt: DT.as_secs_f32() / 100.0,
            ..Default::default()
        };

        self.physics_pipeline.step(
            &vector![0.0, 0.0],
            &integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.hulls.bodies,
            &mut self.hulls.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &Hooks,
            &self.events,
        );

        // Sync back to hulls.
        for hull in self.hulls.hulls.values_mut() {
            let body = &self.hulls.bodies[hull.rb];
            hull.position = body.position().translation.vector / PHYSIC_SCALE;
            hull.rotation = body.position().rotation;
            hull.linvel = *body.linvel() / PHYSIC_SCALE;
            hull.angvel = body.angvel();
        }

        // TODO Handle physic events.

        // Update hulls.
        let mut i = 0;
        let mut hull = Hull::default();
        while i < self.hulls.hulls.len() {
            let v = self.hulls.hulls.get_index_mut(i).unwrap();
            std::mem::swap(&mut hull, v.1);
            let hull_id = *v.0;
            drop(v);

            if let Some(reason) = hull.update(hull_id, &mut self.hulls) {
                hull.on_remove(reason);

                self.hulls.hulls.swap_remove_index(i);

                // Remove its body and collider.
                self.hulls.bodies.remove(
                    hull.rb,
                    &mut self.islands,
                    &mut self.hulls.colliders,
                    &mut self.impulse_joints,
                    &mut self.multibody_joints,
                    true,
                );
            } else {
                let v = self.hulls.hulls.get_index_mut(i).unwrap();
                std::mem::swap(&mut hull, v.1);
                i += 1;
            }
        }
    }

    // /// group_ignore: Any entity in the same group ignore will not interact.
    // /// Can only have one.
    // pub fn add_body(
    //     &mut self,
    //     position: Isometry2<f32>,
    //     linvel: Vector2<f32>,
    //     angvel: f32,

    //     data: EntityDataId,

    //     entity_id: EntityId,
    //     group_ignore: u64,
    // ) -> RigidBodyHandle {
    //     let rb = RigidBodyBuilder::dynamic()
    //         .position(position)
    //         .linvel(linvel)
    //         .angvel(angvel)
    //         .user_data(UserData::pack_body(entity_id, group_ignore))
    //         .linear_damping(DEFAULT_LINEAR_DAMPING)
    //         .angular_damping(DEFAULT_ANGULAR_DAMPING)
    //         .build();
    //     let rb = self.bodies.insert(rb);

    //     let coll = ColliderBuilder::new(data.shape.clone())
    //         .translation(data.shape_translation)
    //         .collision_groups(data.groups)
    //         .mass_properties(data.mprops)
    //         .user_data(UserData::pack_colider(entity_id, false))
    //         .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS)
    //         .active_events(ActiveEvents::CONTACT_FORCE_EVENTS)
    //         .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
    //         .friction(DEFAULT_FRICTION)
    //         .restitution(DEFAULT_RESTITUTION)
    //         .build();

    //     self.colliders
    //         .insert_with_parent(coll, rb, &mut self.bodies);

    //     rb
    // }

    // /// ## Panic:
    // /// Handle is invalid.
    // pub fn remove_collider(&mut self, handle: ColliderHandle) -> Collider {
    //     self.colliders
    //         .remove(handle, &mut self.islands, &mut self.bodies, false)
    //         .unwrap()
    // }

    // // pub fn intersect_broad(&self, aabb: &Aabb) {
    // //     self.query_pipeline.colliders_with_aabb_intersecting_aabb(aabb, callback)
    // // }

    // // pub fn intersection_with_shape(
    // //     &self,
    // //     shape_pos: &Isometry<Real>,
    // //     shape: &dyn Shape,
    // //     filter: QueryFilter,
    // // ) -> Option<ColliderHandle> {
    // //     self.query_pipeline.intersection_with_shape(
    // //         &self.bodies,
    // //         &self.colliders,
    // //         shape_pos,
    // //         shape,
    // //         filter,
    // //     )
    // // }
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

    pub with_entity_id: HullId,
    pub with_shield: bool,

    /// The world-space point of the force with strongest magnitude.
    pub point: Point2<f32>,

    /// The world-space (unit) direction of the force with strongest magnitude.
    pub force_direction: Vector2<f32>,
    /// The magnitude of the largest force at a contact point of this contact pair.
    pub force_magnitude: f32,
}

#[derive(Default)]
pub struct PhysicsEventCollector(pub Arc<Mutex<Vec<(HullId, ContactEvent)>>>);
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
        // // TODO: only take event when force at a point is > some value
        // // contact_pair.
        // let mut point1 = Point2::default();
        // log::debug!("num contact manifold: {}", contact_pair.manifolds.len());
        // for m in contact_pair.manifolds.iter() {
        //     // m.points
        //     log::debug!("contact points: {:?}", m.points);
        //     for p in m.points.iter() {
        //         point1 = p.local_p1;
        //     }
        // }

        // let event = ContactForceEvent::from_contact_pair(dt, contact_pair, total_force_magnitude);

        // let a = colliders[contact_pair.collider1].user_data;
        // let b = colliders[contact_pair.collider2].user_data;

        // let entity_id = a.entity_id();
        // let event = ContactEvent {
        //     shield: false,

        //     with_entity_id: b.entity_id(),
        //     with_shield: false,

        //     point: point1,
        //     force_direction: event.max_force_direction,
        //     force_magnitude: event.max_force_magnitude,
        // };

        // self.0.try_lock().unwrap().push((entity_id, event));
    }
}

/// Body:
/// - HullId: u64
/// - Group ignore: 64
/// Collider:
/// - HullId: u64
/// - Is shield: 1
pub trait UserData {
    fn pack_body(hull_id: HullId, group_ignore: u64) -> Self;
    fn pack_colider(hull_id: HullId, shield: bool) -> Self;

    fn set_group_ignore(&mut self, group_ignore: u64);

    fn hull_id(self) -> HullId;
    fn group_ignore(self) -> u64;
    fn is_shield(self) -> bool;
}
impl UserData for u128 {
    fn pack_body(hull_id: HullId, group_ignore: u64) -> Self {
        hull_id.to_u64() as u128 | (group_ignore as u128) << 64
    }

    fn pack_colider(hull_id: HullId, shield: bool) -> Self {
        hull_id.to_u64() as u128 | (shield as u128) << 64
    }

    fn set_group_ignore(&mut self, group_ignore: u64) {
        *self = (*self & u64::MAX as u128) | (group_ignore as u128) << 64;
    }

    fn hull_id(self) -> HullId {
        HullId::try_from_u64(self as u64).unwrap()
    }

    fn group_ignore(self) -> u64 {
        (self >> 64) as u64
    }

    fn is_shield(self) -> bool {
        self >> 64 != 0
    }
}
