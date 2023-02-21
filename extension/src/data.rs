use crate::client_battlescape::EntityRenderData;
use std::hash::Hash;

use super::battlescape::entity::*;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct EntityDataId(u32);
impl EntityDataId {
    pub fn data(self) -> &'static EntityData {
        &Data::data().entities[self.0 as usize]
    }

    pub fn render_data(self) -> &'static EntityRenderData {
        &Data::data().entities_render[self.0 as usize]
    }
}

pub struct ShipDataIter {
    next_id: u32,
    len: u32,
}
impl Iterator for ShipDataIter {
    type Item = ShipDataId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_id < self.len {
            let id = ShipDataId(self.next_id);
            self.next_id += 1;
            Some(id)
        } else {
            None
        }
    }
}
pub fn ship_data_iter() -> ShipDataIter {
    ShipDataIter {
        next_id: 0,
        len: Data::data().ships.len() as u32,
    }
}

#[derive(Debug)]
pub struct Data {
    pub ships_path: AHashMap<String, ShipDataId>,
    pub ships: Vec<ShipData>,

    pub entities_path: AHashMap<String, EntityDataId>,
    pub entities: Vec<EntityData>,
    pub entities_render: Vec<EntityRenderData>,
}
impl Data {
    /// Free all resources.
    /// ## Safety:
    /// Data should not be in use.
    pub fn clear() {
        unsafe {
            DATA = None;
        }
    }

    pub fn add_ship(path: String, ship_data: ShipData) -> ShipDataId {
        let data = Self::data_mut();
        let id = ShipDataId(data.ships.len() as u32);
        data.ships_path.insert(path, id);
        data.ships.push(ship_data);
        id
    }

    pub fn add_entity(
        path: String,
        entity_data: EntityData,
        entity_render_data: EntityRenderData,
    ) -> EntityDataId {
        let data = Self::data_mut();
        let id = EntityDataId(data.entities.len() as u32);
        data.entities_path.insert(path, id);
        data.entities.push(entity_data);
        data.entities_render.push(entity_render_data);
        id
    }

    pub fn data() -> &'static Data {
        unsafe { DATA.get_or_insert_default() }
    }

    fn data_mut() -> &'static mut Data {
        unsafe { DATA.get_or_insert_default() }
    }
}
impl Default for Data {
    fn default() -> Self {
        Self {
            ships_path: Default::default(),
            ships: Default::default(),
            entities_path: Default::default(),
            entities: Default::default(),
            entities_render: Default::default(),
        }
    }
}

static mut DATA: Option<Data> = None;
