use crate::compressed_vec2::CVec2;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::connection::Connection;

pub trait Asd {
    fn prepare_entities_states(
        &mut self,
        tick: u32,
        client_entity_position: Vec2,
        num_relative_entities_position: u8,
    ) -> bool;
}

impl Asd for Connection {
    fn prepare_entities_states(
        &mut self,
        tick: u32,
        client_entity_position: Vec2,
        num_relative_entities_position: u8,
    ) -> bool {
        if self.can_write(14 + num_relative_entities_position as usize * 6) {
            false
        } else {
            self.write_u8(0);
            self.write_u32(tick);
            self.write_vec2(client_entity_position);
            self.write_u8(num_relative_entities_position);
            true
        }
    }
}

/// `u8` - packet type
///
/// `u32` - `tick`
///
/// `(f32, f32)` - `client_entity_position`
///
/// `u8 [(u16, (u16, u16))]` - `relative_entities_position`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesStates {
    pub tick: u32,
    pub client_entity_position: Vec2,
    /// Entity's id and position compressed and relative to client's position.
    ///
    /// Compressed to 16 + 32 bits
    pub relative_entities_position: Vec<(u16, CVec2)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    /// Server send some entities's position.
    EntitiesStates(EntitiesStates),
}
impl Packet {
    /// Deserialize into an usable packet and return how many bytes were read.
    pub fn deserialize(buf: &[u8]) -> Option<(Self, usize)> {
        if let Some((t, rest)) = buf.split_first() {
            match t.to_owned() {
                0 => {
                    if rest.len() < 13 {
                        return None;
                    }

                    let (tick, rest) = rest.split_array_ref();
                    let tick = u32::from_be_bytes(*tick);

                    let (x, rest) = rest.split_array_ref();
                    let (y, rest) = rest.split_array_ref();
                    let client_entity_position = Vec2::new(f32::from_be_bytes(*x), f32::from_be_bytes(*y));

                    let (len, rest) = rest.split_first().unwrap();
                    let len = *len as usize * 6;

                    if len > rest.len() {
                        return None;
                    }

                    let relative_entities_position = if len != 0 {
                        unsafe {
                            let vs: &[[u8; 6]] = rest[..len].as_chunks_unchecked();
                            vs.iter()
                                .map(|v| {
                                    let v = v.as_chunks_unchecked::<2>();
                                    (
                                        u16::from_be_bytes(v[0]),
                                        CVec2::new(u16::from_be_bytes(v[1]), u16::from_be_bytes(v[2])),
                                    )
                                })
                                .collect()
                        }
                    } else {
                        Vec::new()
                    };

                    Some((
                        Self::EntitiesStates(EntitiesStates {
                            tick,
                            client_entity_position,
                            relative_entities_position,
                        }),
                        1 + 4 + 8 + 1 + len,
                    ))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
