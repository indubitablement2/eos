use super::*;
use bytes::{Buf, BufMut};

pub struct ClientConnection {
    pub client_id: ClientId,
    pub connection: Connection,
    pub token: u64,

    pub view: ClientView,
}
impl ClientConnection {
    pub fn new(connection: Connection, client_id: ClientId) -> Self {
        Self {
            client_id,
            connection,
            token: random(),
            view: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub enum ClientView {
    #[default]
    None,
    Metascape {
        metascape_id: MetascapeId,
        fleet: FleetId,
        knows_fleets: AHashMap<FleetId, KnownFleet>,
    },
    Battlescape {
        battlescape_id: BattlescapeId,
    },
}

#[derive(Debug, Default)]
pub struct KnownFleet {
    pub full_info: bool,
}

#[derive(Debug)]
pub enum ServerPacket {
    LoginResponse {
        token: u64,
        success: bool,
    },
    State {
        time: f64,
        partial_fleets_info: Vec<(FleetId, u32)>,
        full_fleets_info: Vec<(FleetId, u32)>,
        positions: Vec<(FleetId, Vector2<f32>)>,
        remove_fleets: Vec<FleetId>,
    },
    PracticeBattlescapeCreated {
        battlescape_id: BattlescapeId,
        instance_addr: String,
    },
}
impl SerializePacket for ServerPacket {
    fn serialize(self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            ServerPacket::LoginResponse { token, success } => {
                buf.reserve_exact(8 + 4);
                buf.put_u64_le(token);
                buf.put_u32_le(success as u32);
            }
            ServerPacket::State {
                time,
                partial_fleets_info,
                full_fleets_info,
                positions,
                remove_fleets,
            } => {
                buf.reserve_exact(
                    4 + 8
                        + 4
                        + 4
                        + 4
                        + 4
                        + partial_fleets_info.len() * (8 + 4)
                        + full_fleets_info.len() * (8 + 4)
                        + positions.len() * (8 + 4 + 4)
                        + remove_fleets.len() * 8,
                );

                buf.put_u32_le(1);
                buf.put_f64_le(time);

                buf.put_u32_le(partial_fleets_info.len() as u32);
                buf.put_u32_le(full_fleets_info.len() as u32);
                buf.put_u32_le(positions.len() as u32);
                buf.put_u32_le(remove_fleets.len() as u32);

                for (id, num_ship) in partial_fleets_info {
                    buf.put_u64_le(id.0);
                    buf.put_u32_le(num_ship);
                }

                for (id, num_ship) in full_fleets_info {
                    buf.put_u64_le(id.0);
                    buf.put_u32_le(num_ship);
                }

                for (id, position) in positions {
                    buf.put_u64_le(id.0);
                    buf.put_f32_le(position.x);
                    buf.put_f32_le(position.y);
                }

                for id in remove_fleets {
                    buf.put_u64_le(id.0);
                }
            }
            ServerPacket::PracticeBattlescapeCreated {
                battlescape_id,
                instance_addr,
            } => {
                buf.reserve_exact(4 + 8 + instance_addr.len());

                buf.put_u32_le(2);
                buf.put_u64_le(battlescape_id.0);
                buf.put(instance_addr.as_bytes());
            }
        }

        buf
    }
}

#[derive(Debug)]
pub enum ClientPacket {
    // 4 packet_id
    // 4 metascape_id
    // 8 fleet_id
    // 8 wish_position
    MoveFleet {
        metascape_id: MetascapeId,
        fleet_id: FleetId,
        wish_position: Vector2<f32>,
    },
    // 4 packet_id
    CreatePracticeBattlescape,
}
impl DeserializePacket for ClientPacket {
    fn deserialize(mut buf: &[u8]) -> Option<Self> {
        if buf.remaining() < 4 {
            log::debug!("Received ClientPacket of size < 4");
            return None;
        }
        let packet_id = buf.get_u32_le();

        match packet_id {
            0 => {
                if buf.remaining() < 4 + 8 + 8 {
                    log::debug!("Invalid packet size for MoveFleet");
                    return None;
                }
                let metascape_id = MetascapeId(buf.get_u32_le());
                let fleet_id = FleetId(buf.get_u64_le());
                let wish_position = Vector2::new(buf.get_f32_le(), buf.get_f32_le());
                Some(Self::MoveFleet {
                    metascape_id,
                    fleet_id,
                    wish_position,
                })
            }
            1 => Some(ClientPacket::CreatePracticeBattlescape),
            _ => {
                log::debug!("Invalid packet id {}", packet_id);
                None
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginPacket {
    pub username: Option<String>,
    pub password: Option<String>,
}
impl DeserializePacket for LoginPacket {
    fn deserialize(packet: &[u8]) -> Option<Self> {
        match serde_json::from_slice(packet) {
            Ok(packet) => Some(packet),
            Err(err) => {
                log::debug!("Failed to deserialize LoginPacket: {}", err);
                None
            }
        }
    }
}
