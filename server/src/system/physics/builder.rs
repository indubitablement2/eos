use super::*;

const DEFAULT_LINEAR_DAMPING: f32 = 0.01;
const DEFAULT_ANGULAR_DAMPING: f32 = 0.01;

const DEFAULT_FRICTION: f32 = 0.4;
const DEFAULT_RESTITUTION: f32 = 0.1;

const GROUP_SHIP: Group = Group::GROUP_1;
const GROUP_SHIELD: Group = Group::GROUP_2;
const GROUP_DEBRIS: Group = Group::GROUP_3;
const GROUP_MISSILE: Group = Group::GROUP_4;
const GROUP_FIGHTER: Group = Group::GROUP_5;
const GROUP_PROJECTILE: Group = Group::GROUP_6;
const GROUP_ALL: Group = Group::ALL;

pub struct RigidBodyData {
    pub builder: RigidBodyBuilder,
    pub density: f32,
    pub estimated_radius: f32,
}
impl RigidBodyData {
    pub fn build(&self) -> RigidBody {
        RigidBodyBuilder::dynamic()
            .linear_damping(DEFAULT_LINEAR_DAMPING)
            .angular_damping(DEFAULT_ANGULAR_DAMPING)
            .additional_mass_properties(MassProperties::from_ball(
                self.density,
                self.estimated_radius,
            ))
            .build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeRawData {
    Ball {
        radius: f32,
    },
    Cuboid {
        hx: f32,
        hy: f32,
    },
    Compound {
        convex_hulls: Vec<(Isometry2<f32>, Vec<Point<Real>>)>,
    },
}
impl ShapeRawData {
    pub fn shape(&self) -> SharedShape {
        match self {
            ShapeRawData::Ball { radius } => SharedShape::ball(*radius),
            ShapeRawData::Cuboid { hx, hy } => SharedShape::cuboid(*hx, *hy),
            ShapeRawData::Compound { convex_hulls } => {
                let mut shapes = convex_hulls
                    .iter()
                    .map(|(position, points)| {
                        (*position, SharedShape::convex_hull(&points).unwrap())
                    })
                    .collect::<Vec<_>>();

                if shapes.len() == 1 {
                    shapes.pop().unwrap().1
                } else {
                    SharedShape::compound(shapes)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColliderType {
    Ship,
    Missile,
    Fighter,
    Projectile,
}
impl ColliderType {
    pub fn collision_groups(self) -> InteractionGroups {
        match self {
            ColliderType::Ship => InteractionGroups::new(GROUP_SHIP, GROUP_ALL),
            ColliderType::Missile => InteractionGroups::new(GROUP_MISSILE, GROUP_ALL),
            ColliderType::Fighter => {
                InteractionGroups::new(GROUP_FIGHTER, GROUP_MISSILE | GROUP_PROJECTILE)
            }
            ColliderType::Projectile => InteractionGroups::new(
                GROUP_PROJECTILE,
                GROUP_SHIP | GROUP_SHIELD | GROUP_DEBRIS | GROUP_MISSILE | GROUP_FIGHTER,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColliderData {
    pub initial_angle: f32,
    pub initial_translation: Vector2<f32>,
    // pub density: f32,
    pub shape_raw_data: ShapeRawData,
    pub collision_type: ColliderType,
    pub contact_force_event_threshold: f32,
}
impl ColliderData {
    pub fn build(&self) -> Collider {
        ColliderBuilder::new(self.shape_raw_data.shape())
            .collision_groups(self.collision_type.collision_groups())
            .active_hooks(ActiveHooks::FILTER_INTERSECTION_PAIR)
            .active_events(ActiveEvents::CONTACT_FORCE_EVENTS)
            .contact_force_event_threshold(self.contact_force_event_threshold)
            .friction(DEFAULT_FRICTION)
            .restitution(DEFAULT_RESTITUTION)
            .rotation(self.initial_angle)
            .translation(self.initial_translation)
            .density(0.0)
            .build()
    }
}
