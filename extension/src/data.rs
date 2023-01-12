use super::battlescape::entity::EntityData;
use super::metascape::ship::ShipData;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShipDataId(u32);
impl ShipDataId {
    pub fn data(self) -> &'static ShipData {
        &Data::data().ships[self.0 as usize]
    }
}

/// An entity that can be spawned in a battlescape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityDataId(u32);
impl EntityDataId {
    pub fn data(self) -> &'static EntityData {
        &Data::data().entities[self.0 as usize]
    }
}

pub struct Data {
    ships: Vec<ShipData>,
    entities: Vec<EntityData>,
}
impl Data {
    /// ## Safety:
    /// Data should not be in use.
    pub fn reset() {
        unsafe {
            DATA = Some(Default::default());
        }
    }

    pub fn load_file(path: &str) {
        // TODO:
    }

    pub fn add_entity_data(&mut self, entity_data: EntityData) -> EntityDataId {
        let id = EntityDataId(self.entities.len() as u32);
        self.entities.push(entity_data);
        id
    }

    fn data() -> &'static Data {
        unsafe { DATA.get_or_insert_default() }
    }

    fn data_mut() -> &'static mut Data {
        unsafe { DATA.get_or_insert_default() }
    }
}
impl Default for Data {
    fn default() -> Self {
        Self {
            ships: vec![ShipData {
                entity_data_id: EntityDataId(0),
            }],
            entities: vec![EntityData::default()],
        }
    }
}

static mut DATA: Option<Data> = None;
