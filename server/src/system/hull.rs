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
        let u = self.entity_idx as u32 | ((self.idx as u32) << 16);
        serializer.serialize_u32(u)
    }
}
impl<'de> serde::Deserialize<'de> for &'static HullData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let u = u32::deserialize(deserializer)?;
        let entity_idx = (u & 0xffff) as usize;
        let idx = (u >> 16) as usize;
        Ok(&Data::data().entities[entity_idx].hulls[idx])
    }
}

pub struct HullData {
    pub entity_idx: u16,
    pub idx: u16,

    pub collider: Collider,
    pub initial_position: Isometry<f32>,

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
