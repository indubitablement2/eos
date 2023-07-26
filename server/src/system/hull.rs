use super::*;

#[derive(Serialize, Deserialize)]
pub struct Hull {
    pub data: &'static HullData,
    pub collider: ColliderHandle,
}
impl Hull {
    // pub fn new(data: HullDataId) -> Self {
    //     // let builder = SimpleColliderBuilder::n
    //     todo!()
    // }

    pub fn step(&mut self) {
        todo!()
    }
}

impl serde::Serialize for &'static HullData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.entity_data_idx)
    }
}
impl<'de> serde::Deserialize<'de> for &'static HullData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entity_data_idx = u16::deserialize(deserializer)? as usize;
        Ok(Data::data().entities[entity_data_idx]
            .hull
            .as_ref()
            .unwrap())
    }
}

pub struct HullData {
    pub entity_data_idx: u16,

    pub collider: Collider,

    pub defence: Defence,
    // todo: engine slot
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: shields
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Defence {
    pub hull: i32,
    pub armor: i32,
}
