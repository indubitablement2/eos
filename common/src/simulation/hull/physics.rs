use super::*;
use parking_lot::Mutex;
use rapier2d::na::{self, Isometry2, Point2, UnitComplex, Vector2};
use rapier2d::prelude::*;
use std::sync::Arc;

const DEFAULT_LINEAR_DAMPING: f32 = 0.01;
const DEFAULT_ANGULAR_DAMPING: f32 = 0.01;
const DEFAULT_FRICTION: f32 = 0.3;
const DEFAULT_RESTITUTION: f32 = 0.2;
const DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD: f32 = 0.0;

/// Shouldn't have to use this.
/// Everything is automatically scaled.
pub const PHYSIC_SCALE: f32 = 1.0 / 64.0;

pub mod group {
    use super::*;

    pub const GROUP_SHIP: Group = Group::GROUP_1;
    pub const GROUP_DEBRIS: Group = Group::GROUP_2;
    pub const GROUP_MISSILE: Group = Group::GROUP_3;
    pub const GROUP_FIGHTER: Group = Group::GROUP_4;

    pub const GROUP_AVOIDANCE_S: Group = Group::GROUP_9;
    pub const GROUP_AVOIDANCE_M: Group = Group::GROUP_10;
    pub const GROUP_AVOIDANCE_L: Group = Group::GROUP_11;
    pub const GROUP_AVOIDANCE_XL: Group = Group::GROUP_12;
    pub const GROUP_AVOIDANCE_XXL: Group = Group::GROUP_13;
}

// TODO: Move query pipeline here
#[derive(Default)]
pub struct Hulls {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    next_collision_group_ignore: u64,

    next_hull_id: HullId,
    hulls: IndexMap<HullId, Hull>,
}
impl Hulls {
    pub fn insert(&mut self, builder: HullBuilder) -> (HullId, &mut Hull) {
        let hull_id = self.next_hull_id.next();

        let collision_group_ignore = self.next_collision_group_ignore;
        self.next_collision_group_ignore += 1;

        let rb = RigidBodyBuilder::dynamic()
            .position(Isometry2::new(
                builder.position.to_na() * PHYSIC_SCALE,
                builder.rotation,
            ))
            .linvel(builder.linvel.to_na() * PHYSIC_SCALE)
            .angvel(builder.angvel)
            .user_data(UserData::pack_body(hull_id, collision_group_ignore))
            .linear_damping(DEFAULT_LINEAR_DAMPING)
            .angular_damping(DEFAULT_ANGULAR_DAMPING)
            .build();
        let rb = self.bodies.insert(rb);

        let coll = ColliderBuilder::new(builder.hull_data_id.shape.clone())
            .position(builder.hull_data_id.shape_position)
            .collision_groups(builder.hull_data_id.groups)
            .mass_properties(builder.hull_data_id.mprops)
            .user_data(UserData::pack_colider(hull_id, false))
            .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS)
            .active_events(ActiveEvents::CONTACT_FORCE_EVENTS)
            .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
            .friction(DEFAULT_FRICTION)
            .restitution(DEFAULT_RESTITUTION)
            .build();
        self.colliders
            .insert_with_parent(coll, rb, &mut self.bodies);

        let hull = Hull {
            hull_data_id: builder.hull_data_id,
            ship_id: builder.ship_id,
            owner: builder.owner,
            rb,
            position: builder.position,
            rotation: builder.rotation,
            linvel: builder.linvel,
            angvel: builder.angvel,
            collision_group_ignore,
            hull_max_percent_increase: 0,
            hull_max_flat_increase: 0,
            hull_relative: builder.hull_relative,
            armor_max_percent_increase: 0,
            armor_max_flat_increase: 0,
            armor_cells: (),
            linacc_percent_increase: 0,
            linacc_flat_increase: 0,
            angacc_percent_increase: 0,
            angacc_flat_increase: 0,
            linvel_max_percent_increase: 0,
            linvel_max_flat_increase: 0,
            angvel_max_percent_increase: 0,
            angvel_max_flat_increase: 0,
            wish_angvel: Default::default(),
            wish_linvel: Default::default(),
            controlled: false,
            target: None,
            modifiers: Default::default(),
        };

        let hull = self.hulls.entry(hull_id).or_insert(hull);

        for modifier in builder.modifiers {
            modifier.apply(hull);
        }

        hull.on_new();

        (hull_id, hull)
    }

    pub fn contains(&self, hull_id: HullId) -> bool {
        self.hulls.contains_key(&hull_id)
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

// TODO: Remove shield from physics. use broccoli
/// All bodies/colliders are hulls. 1 collider per body.
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
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    events: PhysicsEventCollector,
}
impl Physics {
    pub fn step(&mut self, clients: &mut Clients) {
        // Sync from hulls.
        for hull in self.hulls.hulls.values() {
            let body = &mut self.hulls.bodies[hull.rb];
            body.set_position(
                Isometry2::new(hull.position.to_na() * PHYSIC_SCALE, hull.rotation),
                true,
            );
            body.set_linvel(hull.linvel.to_na() * PHYSIC_SCALE, true);
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
            hull.position = body.position().translation.vector.to_glam() / PHYSIC_SCALE;
            hull.rotation = body.position().rotation.angle();
            hull.linvel = body.linvel().to_glam() / PHYSIC_SCALE;
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

            if let Some(reason) = hull.on_update(hull_id, &mut self.hulls, clients) {
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

pub trait Vec2ToNa {
    fn to_na(self) -> Vector2<f32>;
}
impl Vec2ToNa for Vec2 {
    fn to_na(self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
}

trait Vec2ToGlam {
    fn to_glam(self) -> Vec2;
}
impl Vec2ToGlam for Vector2<f32> {
    fn to_glam(self) -> Vec2 {
        Vec2::new(self.x, self.y)
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
