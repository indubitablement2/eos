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
}

pub struct ShipDataIter {
    next_id: u32,
    len: u32,
}
impl Iterator for ShipDataIter {
    type Item = (ShipDataId, &'static ShipData);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_id < self.len {
            let id = ShipDataId(self.next_id);
            self.next_id += 1;
            Some((id, id.data()))
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
    pub ships: IndexMap<String, ShipData, RandomState>,
    pub entities: IndexMap<String, EntityData, RandomState>,
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
        ShipDataId(Self::data_mut().ships.insert_full(path, ship_data).0 as u32)
    }

    pub fn add_entity(path: String, entity_data: EntityData) -> EntityDataId {
        EntityDataId(Self::data_mut().entities.insert_full(path, entity_data).0 as u32)
    }

    pub fn ship_data_from_path(path: String) -> Option<(ShipDataId, &'static ShipData)> {
        Self::data()
            .ships
            .get_full(&path)
            .map(|(idx, _, data)| (ShipDataId(idx as u32), data))
    }

    pub fn entity_data_from_path(path: String) -> Option<(EntityDataId, &'static EntityData)> {
        Self::data()
            .entities
            .get_full(&path)
            .map(|(idx, _, data)| (EntityDataId(idx as u32), data))
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
            ships: Default::default(),
            entities: Default::default(),
        }
    }
}

static mut DATA: Option<Data> = None;
