use super::*;
use bytes::{Buf, BufMut};

pub struct ClientConnection {
    pub client_id: ClientId,
    pub connection: Connection<ClientPacket, ServerPacket>,

    pub knows_fleets: AHashMap<FleetId, KnownFleet>,
    pub view: (MetascapeId, Vector2<f32>),
}

pub struct KnownFleet {
    pub full_info: bool,
}

#[derive(Debug)]
pub enum ServerPacket {
    LoginResponse {
        success: bool,
        reason: Option<String>,
    },
    State {
        time: f64,
        partial_fleets_info: Vec<(FleetId, u32)>,
        full_fleets_info: Vec<(FleetId, u32)>,
        positions: Vec<(FleetId, Vector2<f32>)>,
        remove_fleets: Vec<FleetId>,
    },
}
impl SerializePacket for ServerPacket {
    fn serialize(self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            ServerPacket::LoginResponse { success, reason } => {
                buf.reserve_exact(
                    4 + 4 + reason.as_ref().map(|reason| reason.len() + 4).unwrap_or(0),
                );

                buf.put_u32_le(0);
                buf.put_u32_le(success as u32);
                if let Some(reason) = reason {
                    buf.put_u32_le(reason.len() as u32);
                    buf.put(reason.as_bytes());
                }
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
        }

        buf
    }
}

#[derive(Debug)]
pub enum ClientPacket {
    Login(LoginPacket),
    MetascapeCommand {
        metascape_id: MetascapeId,
        cmd: MetascapeCommand,
    },
}
impl DeserializePacket for ClientPacket {
    fn deserialize(mut buf: &[u8]) -> Option<Self> {
        if buf.remaining() < 4 {
            log::debug!("Received ClientPacket of size < 4");
            return None;
        }
        let packet_id = buf.get_u32_le();

        match packet_id {
            0 => match serde_json::from_slice(buf) {
                Ok(login_packet) => Some(Self::Login(login_packet)),
                Err(err) => {
                    log::debug!("Invalid LoginPacket: {}", err);
                    None
                }
            },
            1 => {
                if buf.remaining() < 8 {
                    log::debug!("Invalid packet size for MetascapeCommand");
                    return None;
                }
                let metascape_id = MetascapeId(buf.get_u32_le());
                let cmd_id = buf.get_u32_le();

                match cmd_id {
                    0 => {
                        if buf.remaining() < 8 + 8 {
                            log::debug!("Invalid packet size for MoveFleet");
                            return None;
                        }
                        let fleet_id = FleetId(buf.get_u64_le());
                        let wish_position = Vector2::new(buf.get_f32_le(), buf.get_f32_le());

                        // 4 packet_id
                        // 4 metascape_id
                        // 4 cmd_id
                        // 8 fleet_id
                        // 8 wish_position
                        Some(ClientPacket::MetascapeCommand {
                            metascape_id,
                            cmd: MetascapeCommand::MoveFleet {
                                fleet_id,
                                wish_position,
                            },
                        })
                    }
                    _ => {
                        log::debug!("Invalid cmd id {}", cmd_id);
                        None
                    }
                }
            }
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
