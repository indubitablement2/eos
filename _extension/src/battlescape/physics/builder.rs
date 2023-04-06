use super::*;
use std::ops::Deref;

pub enum Groups {
    Ship,
}
impl Groups {
    fn groups(self) -> InteractionGroups {
        match self {
            Groups::Ship => InteractionGroups::new(GROUP_SHIP, GROUP_ALL),
        }
    }
}

pub fn ball_collider(radius: f32, density: f32, groups: Groups) -> Collider {
    build_collider(SharedShape::ball(radius), density, groups)
}

pub fn cuboid_collider(hx: f32, hy: f32, density: f32, groups: Groups) -> Collider {
    build_collider(SharedShape::cuboid(hx, hy), density, groups)
}

pub fn polygons_collider(
    mut polygons: Vec<Vec<na::Point2<f32>>>,
    density: f32,
    groups: Groups,
) -> Collider {
    let shape = if polygons.is_empty() {
        log::warn!("Polygons must have at least 1 polygon. Using ball instead...");
        return ball_collider(0.5, density, groups);
    } else if polygons.len() == 1 {
        SharedShape::convex_polyline(polygons.pop().unwrap()).unwrap()
    } else {
        SharedShape::compound(
            polygons
                .into_iter()
                .map(|points| {
                    (
                        na::Isometry2::default(),
                        SharedShape::convex_polyline(points).unwrap(),
                    )
                })
                .collect(),
        )
    };

    build_collider(shape, density, groups)
}

fn build_collider(shape: SharedShape, density: f32, groups: Groups) -> Collider {
    let mut mass_properties = ColliderMassProps::Density(density).mass_properties(shape.deref());
    mass_properties.local_com = Default::default();

    ColliderBuilder::new(shape)
        .collision_groups(groups.groups())
        // TODO: Need ActiveHooks::FILTER_INTERSECTION_PAIR ?
        .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS)
        .active_events(ActiveEvents::all())
        .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
        .friction(DEFAULT_FRICTION)
        .restitution(DEFAULT_RESTITUTION)
        .mass_properties(mass_properties)
        .build()
}

// pub struct SimpleRigidBodyBuilder {
//     pub builder: RigidBodyBuilder,
//     pub copy_group_ignore: Option<RigidBodyHandle>,
// }
// impl SimpleRigidBodyBuilder {
//     pub fn dynamic() -> Self {
//         Self {
//             builder: RigidBodyBuilder::dynamic()
//                 .linear_damping(DEFAULT_LINEAR_DAMPING)
//                 .angular_damping(DEFAULT_ANGULAR_DAMPING)
//                 .can_sleep(false),
//             copy_group_ignore: None,
//         }
//     }

//     pub fn kinematic_position_based() -> Self {
//         Self {
//             builder: RigidBodyBuilder::kinematic_position_based(),
//             copy_group_ignore: None,
//         }
//     }

//     pub fn translation(mut self, translation: na::Vector2<f32>) -> Self {
//         self.builder = self.builder.translation(translation);
//         self
//     }

//     pub fn rotation(mut self, angle: f32) -> Self {
//         self.builder = self.builder.rotation(angle);
//         self
//     }

//     pub fn linvel(mut self, linvel: na::Vector2<f32>) -> Self {
//         self.builder = self.builder.linvel(linvel);
//         self
//     }

//     pub fn angvel(mut self, angvel: f32) -> Self {
//         self.builder = self.builder.angvel(angvel);
//         self
//     }

//     pub fn copy_group_ignore(mut self, from: RigidBodyHandle) -> Self {
//         self.copy_group_ignore = Some(from);
//         self
//     }
// }

// pub struct SimpleColliderBuilder {
//     pub builder: ColliderBuilder,
// }
// impl SimpleColliderBuilder {
//     pub const GROUP_SHIP: Group = Group::GROUP_1;
//     pub const GROUP_SHIELD: Group = Group::GROUP_2;
//     pub const GROUP_DEBRIS: Group = Group::GROUP_3;
//     pub const GROUP_MISSILE: Group = Group::GROUP_4;
//     pub const GROUP_FIGHTER: Group = Group::GROUP_5;
//     pub const GROUP_PROJECTILE: Group = Group::GROUP_6;

//     pub const GROUP_ALL: Group = Self::GROUP_SHIP
//         .union(Self::GROUP_SHIELD)
//         .union(Self::GROUP_DEBRIS)
//         .union(Self::GROUP_MISSILE)
//         .union(Self::GROUP_FIGHTER)
//         .union(Self::GROUP_PROJECTILE);

//     pub const GROUPS_SHIP: InteractionGroups =
//         InteractionGroups::new(Self::GROUP_SHIP, Self::GROUP_ALL);

//     pub fn new_ship(shape: SharedShape) -> Self {
//         Self {
//             builder: ColliderBuilder::new(shape)
//                 .collision_groups(Self::GROUPS_SHIP)
//                 // Need ActiveHooks::FILTER_INTERSECTION_PAIR ?
//                 .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS)
//                 .active_events(ActiveEvents::all())
//                 .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
//                 .friction(DEFAULT_FRICTION)
//                 .restitution(DEFAULT_RESTITUTION),
//         }
//     }

//     /// Sets the translation relative to the rigid-body it will be attached to.
//     pub fn translation(mut self, translation: na::Vector2<f32>) -> Self {
//         self.builder = self.builder.translation(translation);
//         self
//     }

//     /// Sets the orientation relative to the rigid-body it will be attached to.
//     pub fn rotation(mut self, angle: f32) -> Self {
//         self.builder = self.builder.rotation(angle);
//         self
//     }

//     pub fn position(mut self, pos: Isometry<Real>) -> Self {
//         self.builder = self.builder.position(pos);
//         self
//     }

//     pub fn density(mut self, density: f32) -> Self {
//         self.builder = self.builder.density(density);
//         self
//     }
// }
