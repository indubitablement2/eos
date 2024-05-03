use super::*;

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

#[derive(Debug, Serialize, Deserialize, Default)]
struct EntityDataJson {
    hull: f32,

    armor_max: f32,
    armor_cells_offset: Vector2<f32>,
    armor_cells_size: Vector2<i32>,
    armor_cells: Vec<f32>,

    shape_translation: Vector2<f32>,
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

            hull_max: self.hull,

            armor_max: self.armor_max,
            armor_cells_offset: self.armor_cells_offset,
            armor_cells: (),
            // self
            //     .armor_cells
            //     .into_iter()
            //     .map(|v| (v * u8::MAX as f32) as u8)
            //     .collect(),
            shape_translation: self.shape_translation,
            shape: self.shape.to_shared_shape(),
            mprops: MassProperties::from_ball(self.density, self.mass_radius),
            groups: InteractionGroups {
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
    Polygon { vertices: Vec<Point<f32>> },
}
impl HullShapeJson {
    fn to_shared_shape(&self) -> SharedShape {
        match self {
            HullShapeJson::Cuboid { hx, hy } => SharedShape::cuboid(*hx, *hy),
            HullShapeJson::Ball { radius } => SharedShape::ball(*radius),
            HullShapeJson::Polygon { vertices } => {
                let indices = (0..vertices.len() as u32 - 1)
                    .map(|i| [i, i + 1])
                    .chain(std::iter::once([vertices.len() as u32 - 1, 0]))
                    .collect::<Vec<_>>();
                SharedShape::convex_decomposition(vertices.as_slice(), &indices)
            }
        }
    }
}
impl Default for HullShapeJson {
    fn default() -> Self {
        Self::Ball { radius: 0.5 }
    }
}

#[test]
fn print_json_sample() {
    let json = EntityDataJson {
        hull: 100.0,

        armor_max: 100.0,
        armor_cells_offset: Vector2::new(0.0, 0.0),
        armor_cells_size: Vector2::new(1, 1),
        armor_cells: vec![1.0],

        shape_translation: Vector2::new(0.0, 0.0),
        shape: HullShapeJson::Ball { radius: 0.5 },
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
