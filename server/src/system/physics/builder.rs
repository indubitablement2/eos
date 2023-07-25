use super::*;

const DEFAULT_LINEAR_DAMPING: f32 = 0.01;
const DEFAULT_ANGULAR_DAMPING: f32 = 0.01;

const DEFAULT_FRICTION: f32 = 0.4;
const DEFAULT_RESTITUTION: f32 = 0.1;
// TODO: How much?
const DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD: f32 = 1.0;

const GROUP_SHIP: Group = Group::GROUP_1;
const GROUP_SHIELD: Group = Group::GROUP_2;
const GROUP_DEBRIS: Group = Group::GROUP_3;
const GROUP_MISSILE: Group = Group::GROUP_4;
const GROUP_FIGHTER: Group = Group::GROUP_5;
const GROUP_PROJECTILE: Group = Group::GROUP_6;
const GROUP_ALL: Group = Group::ALL;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EntityType {
    Ship,
    Missile,
    Fighter,
    Projectile,
}
impl EntityType {
    pub fn from_idx(idx: i32) -> Self {
        match idx {
            0 => EntityType::Ship,
            1 => EntityType::Missile,
            2 => EntityType::Fighter,
            _ => EntityType::Projectile,
        }
    }

    pub fn interaction_groups(self) -> InteractionGroups {
        match self {
            EntityType::Ship => InteractionGroups::new(GROUP_SHIP, GROUP_ALL),
            EntityType::Missile => InteractionGroups::new(GROUP_MISSILE, GROUP_ALL),
            EntityType::Fighter => {
                InteractionGroups::new(GROUP_FIGHTER, GROUP_MISSILE | GROUP_PROJECTILE)
            }
            EntityType::Projectile => InteractionGroups::new(
                GROUP_PROJECTILE,
                GROUP_SHIP | GROUP_SHIELD | GROUP_DEBRIS | GROUP_MISSILE | GROUP_FIGHTER,
            ),
        }
    }
}

pub fn make_rigid_body(density: f32, estimated_radius: f32) -> RigidBody {
    let mprops = MassProperties::from_ball(density, estimated_radius);

    RigidBodyBuilder::dynamic()
        .linear_damping(DEFAULT_LINEAR_DAMPING)
        .angular_damping(DEFAULT_ANGULAR_DAMPING)
        .additional_mass_properties(mprops)
        .build()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeRawData {
    Ball { radius: f32 },
    Cuboid { hx: f32, hy: f32 },
    Compound { convex_hulls: Vec<Vec<Point2<f32>>> },
}
impl ShapeRawData {
    pub fn shape(&self) -> SharedShape {
        match self {
            ShapeRawData::Ball { radius } => SharedShape::ball(*radius),
            ShapeRawData::Cuboid { hx, hy } => SharedShape::cuboid(*hx, *hy),
            ShapeRawData::Compound { convex_hulls } => {
                let mut shapes = convex_hulls
                    .iter()
                    .map(|points| {
                        (
                            Isometry2::identity(),
                            SharedShape::convex_hull(&points).unwrap(),
                        )
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

pub fn make_collider(
    initial_position: Isometry2<f32>,
    shape_raw_data: ShapeRawData,
    interaction_groups: InteractionGroups,
) -> Collider {
    ColliderBuilder::new(shape_raw_data.shape())
        .collision_groups(interaction_groups)
        .active_hooks(ActiveHooks::FILTER_INTERSECTION_PAIR)
        .active_events(ActiveEvents::CONTACT_FORCE_EVENTS)
        .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
        .friction(DEFAULT_FRICTION)
        .restitution(DEFAULT_RESTITUTION)
        .position(initial_position)
        .density(0.0)
        .build()
}
