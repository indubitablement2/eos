use crate::compressed_vec2::CVec2;
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesStates {
    pub tick: u32,
    pub client_entity_position: Vec2,
    /// Entity's id and position compressed and relative to client's position.
    pub relative_entities_position: Vec<(u16, CVec2)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Empty,
    Corrupted,
    /// Server send some entities's position.
    EntitiesStates(EntitiesStates),
}
impl Packet {
    /// Serialize a packet into a vector of bytes using bincode.
    pub fn serialize(&self) -> Vec<u8> {
        match bincode::serialize(self) {
            Ok(v) => v,
            Err(err) => {
                error!("{:?} could not serialize packet unsing bincode.", err);
                Vec::new()
            }
        }
    }

    /// Deserialize into an usable packet using bincode.
    pub fn deserialize(buf: &[u8]) -> Self {
        if let Ok(result) = bincode::deserialize(buf) {
            result
        } else {
            if buf.is_empty() {
                Self::Empty
            } else {
                Self::Corrupted
            }
        }
    }
}