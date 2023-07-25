use rapier2d::prelude::InteractionGroups;

use super::*;
use crate::system::{entity::*, hull::*, physics::builder::*};

pub struct Data {
    pub entities: Box<[EntityData]>,
    // pub ships: Vec<ShipData>,
}
impl Data {
    pub fn initialize() {
        unsafe {
            DATA = Some(
                serde_json::from_str::<DataJson>(include_str!(
                    "../../tool/data_editor/server_data.json"
                ))
                .expect("json data should be valid")
                .to_data(),
            );
        }
    }

    pub fn data() -> &'static Self {
        unsafe { DATA.as_ref().expect("data should be initialized") }
    }
}
static mut DATA: Option<Data> = None;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct DataJson {
    entities: Vec<EntityDataJson>,
}
impl DataJson {
    fn to_data(mut self) -> Data {
        self.entities.sort_by_key(|e| e.idx);
        self.entities
            .iter()
            .enumerate()
            .for_each(|(expected_idx, e)| {
                assert!(
                    expected_idx == e.idx as usize,
                    "entity idx should be sequential"
                )
            });

        Data {
            entities: self.entities.into_iter().map(|e| e.to_data()).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConvexHullsJson {
    points_x: Vec<f32>,
    points_y: Vec<f32>,
}
impl ConvexHullsJson {
    pub fn to_data(self) -> Vec<Point2<f32>> {
        self.points_x
            .into_iter()
            .zip(self.points_y.into_iter())
            .map(|(x, y)| Point2::new(x, y))
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Deserialize)]
enum ShapeRawJson {
    Ball { radius: f32 },
    Cuboid { hx: f32, hy: f32 },
    Compound { convex_hulls: Vec<ConvexHullsJson> },
}
impl ShapeRawJson {
    fn to_data(self) -> ShapeRawData {
        match self {
            ShapeRawJson::Ball { radius } => ShapeRawData::Ball { radius },
            ShapeRawJson::Cuboid { hx, hy } => ShapeRawData::Cuboid { hx, hy },
            ShapeRawJson::Compound { convex_hulls } => ShapeRawData::Compound {
                convex_hulls: convex_hulls
                    .into_iter()
                    .map(|value| value.to_data())
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct HullDataJson {
    idx: u16,
    initial_translation_x: f32,
    initial_translation_y: f32,
    initial_angle: f32,
    defence: Defence,

    shape: ShapeRawJson,
}
impl HullDataJson {
    fn to_data(self, entity_idx: u16, interaction_groups: InteractionGroups) -> HullData {
        let initial_position = Isometry2::new(
            Vector2::new(self.initial_translation_x, self.initial_translation_y),
            self.initial_angle,
        );

        let collider = make_collider(initial_position, self.shape.to_data(), interaction_groups);

        HullData {
            entity_idx,
            idx: self.idx,
            collider,
            initial_position,
            defence: self.defence,
        }
    }
}

#[derive(Debug, Deserialize)]
struct EntityDataJson {
    idx: u16,
    density: f32,
    estimated_radius: f32,
    wish_ignore_same_team: bool,
    force_ignore_same_team: bool,
    mobility: Mobility,
    hulls: Vec<HullDataJson>,
    entity_type: i32,
}
impl EntityDataJson {
    fn to_data(mut self) -> EntityData {
        self.hulls.sort_by_key(|h| h.idx);
        self.hulls.iter().enumerate().for_each(|(expected_idx, e)| {
            assert!(
                expected_idx == e.idx as usize,
                "hull idx should be sequential"
            )
        });

        let interaction_groups = EntityType::from_idx(self.entity_type).interaction_groups();

        EntityData {
            idx: self.idx,
            body: make_rigid_body(self.density, self.estimated_radius),
            wish_ignore_same_team: self.wish_ignore_same_team,
            force_ignore_same_team: self.force_ignore_same_team,
            hulls: self
                .hulls
                .into_iter()
                .map(|h| h.to_data(self.idx, interaction_groups))
                .collect(),
            mobility: self.mobility,
        }
    }
}
