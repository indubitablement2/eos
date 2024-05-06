use super::*;
use physics::{Vec2ToNa, PHYSIC_SCALE};

pub fn load_hull_data() {
    let read = std::fs::read("../client/tool/server_data/hulls.json").unwrap();
    let json: Vec<EntityDataJson> = serde_json::from_slice(read.as_slice()).unwrap();
    DATA.set(
        json.into_iter()
            .zip(0u32..)
            .map(|(entity_json, id)| entity_json.parse(id))
            .collect(),
    )
    .ok()
    .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
struct EntityDataJson {
    hull_max: f32,

    armor_max: f32,

    damage_modifier_arcs: Vec<DamageModifierArc>,

    shape_position: Vec2,
    shape_rotation: f32,
    shape: HullShapeJson,
    mass_radius: f32,
    density: f32,
    memberships: u32,
    filter: u32,

    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
    // TODO: Shields
    linear_acceleration: f32,
    angular_acceleration: f32,
    max_linear_velocity: f32,
    max_angular_velocity: f32,

    ai: HullAi,

    on_new: Vec<HullEvent>,
    on_remove: Vec<HullEvent>,
}
impl EntityDataJson {
    fn parse(self, id: u32) -> HullData {
        HullData {
            id,

            hull_max: self.hull_max,

            armor_max: self.armor_max,

            damage_modifier_arcs: self.damage_modifier_arcs,

            shape_position: rapier2d::na::Isometry2::new(
                self.shape_position.to_na() * PHYSIC_SCALE,
                self.shape_rotation,
            ),
            shape: self.shape.to_shared_shape(),
            mprops: rapier2d::dynamics::MassProperties::from_ball(
                self.density,
                self.mass_radius * PHYSIC_SCALE,
            ),
            groups: rapier2d::geometry::InteractionGroups {
                memberships: self.memberships.into(),
                filter: self.filter.into(),
            },

            linacc: self.linear_acceleration,
            angacc: self.angular_acceleration,
            linvel_max: self.max_linear_velocity,
            angvel_max: self.max_angular_velocity,

            ai: self.ai,

            on_new: self.on_new,
            on_remove: self.on_remove,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum HullShapeJson {
    Cuboid { hx: f32, hy: f32 },
    Ball { radius: f32 },
    Polygon { vertices: Vec<Vec2> },
}
impl HullShapeJson {
    fn to_shared_shape(&self) -> rapier2d::geometry::SharedShape {
        match self {
            HullShapeJson::Cuboid { hx, hy } => {
                rapier2d::geometry::SharedShape::cuboid(*hx * PHYSIC_SCALE, *hy * PHYSIC_SCALE)
            }
            HullShapeJson::Ball { radius } => {
                rapier2d::geometry::SharedShape::ball(*radius * PHYSIC_SCALE)
            }
            HullShapeJson::Polygon { vertices } => {
                let indices = (0..vertices.len() as u32 - 1)
                    .map(|i| [i, i + 1])
                    .chain(std::iter::once([vertices.len() as u32 - 1, 0]))
                    .collect::<Vec<_>>();
                let vertices = vertices
                    .iter()
                    .map(|p| rapier2d::na::Point2::new(p.x * PHYSIC_SCALE, p.y * PHYSIC_SCALE))
                    .collect::<Vec<_>>();
                rapier2d::geometry::SharedShape::convex_decomposition(vertices.as_slice(), &indices)
            }
        }
    }
}

#[test]
fn print_json_sample() {
    let json = EntityDataJson {
        hull_max: 100.0,

        armor_max: 100.0,

        damage_modifier_arcs: vec![DamageModifierArc {
            arc_direction: Vec2::Y,
            arc_dot: 1.0,
            modifier: DamageModifier::EngineDamage1_5,
        }],

        shape_position: Vec2::new(123.0, -123.0),
        shape_rotation: 0.5,
        shape: HullShapeJson::Polygon {
            vertices: vec![Vec2::new(-0.5, 1.0), Vec2::new(0.0, -0.5)],
        },
        mass_radius: 0.5,
        density: 1.0,
        memberships: 0,
        filter: 0,

        linear_acceleration: 1.0,
        angular_acceleration: 1.0,
        max_linear_velocity: 1.0,
        max_angular_velocity: 1.0,

        ai: HullAi::default(),

        on_new: vec![],
        on_remove: vec![],
    };

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
