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
const GROUP_PROJECTILE: Group = Group::GROUP_4;
const GROUP_FIGHTER: Group = Group::GROUP_5;
const GROUP_ALL: Group = Group::ALL;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EntityType {
    Ship,
    Fighter,
    Projectile,
}
impl EntityType {
    pub fn from_idx(idx: i32) -> Self {
        match idx {
            0 => EntityType::Ship,
            1 => EntityType::Fighter,
            _ => EntityType::Projectile,
        }
    }

    pub fn interaction_groups(self) -> InteractionGroups {
        match self {
            EntityType::Ship => InteractionGroups::new(GROUP_SHIP, GROUP_ALL),
            EntityType::Projectile => InteractionGroups::new(GROUP_PROJECTILE, GROUP_ALL),
            EntityType::Fighter => InteractionGroups::new(GROUP_FIGHTER, GROUP_PROJECTILE),
        }
    }
}

pub fn make_rigid_body() -> RigidBody {
    RigidBodyBuilder::dynamic()
        .linear_damping(DEFAULT_LINEAR_DAMPING)
        .angular_damping(DEFAULT_ANGULAR_DAMPING)
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
    shape_raw_data: ShapeRawData,
    interaction_groups: InteractionGroups,
    density: f32,
) -> Collider {
    let shape = shape_raw_data.shape();

    let mut mprops = shape.mass_properties(density);
    mprops.local_com = Point2::origin();

    ColliderBuilder::new(shape)
        .collision_groups(interaction_groups)
        .active_hooks(ActiveHooks::FILTER_INTERSECTION_PAIR)
        .active_events(ActiveEvents::CONTACT_FORCE_EVENTS)
        .contact_force_event_threshold(DEFAULT_CONTACT_FORCE_EVENT_THRESHOLD)
        .friction(DEFAULT_FRICTION)
        .restitution(DEFAULT_RESTITUTION)
        .mass_properties(mprops)
        .build()
}
