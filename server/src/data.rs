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
        self.entities.sort_by_key(|e| e.entity_data_idx);
        self.entities
            .iter()
            .enumerate()
            .for_each(|(expected_idx, e)| {
                assert!(
                    expected_idx == e.entity_data_idx as usize,
                    "entity idx should be sequential"
                )
            });

        Data {
            entities: self.entities.into_iter().map(|e| e.to_data()).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct HullDataJson {
    shape: ShapeRawData,
    density: f32,
    defence: Defence,
}
impl HullDataJson {
    fn to_data(self, entity_data_idx: u16, interaction_groups: InteractionGroups) -> HullData {
        HullData {
            entity_data_idx,
            collider: make_collider(self.shape, interaction_groups, self.density),
            defence: self.defence,
        }
    }
}

#[derive(Debug, Deserialize)]
struct EntityDataJson {
    entity_data_idx: u16,
    wish_ignore_same_team: bool,
    force_ignore_same_team: bool,
    mobility: Mobility,
    hull: Option<HullDataJson>,
    entity_type: i32,
}
impl EntityDataJson {
    fn to_data(self) -> EntityData {
        EntityData {
            entity_data_idx: self.entity_data_idx,
            body: make_rigid_body(),
            wish_ignore_same_team: self.wish_ignore_same_team,
            force_ignore_same_team: self.force_ignore_same_team,
            hull: self.hull.map(|h| {
                h.to_data(
                    self.entity_data_idx,
                    EntityType::from_idx(self.entity_type).interaction_groups(),
                )
            }),
            mobility: self.mobility,
        }
    }
}
