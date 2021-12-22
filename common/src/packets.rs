use crate::{idx::*, Version};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV6;

#[derive(Debug, Clone, Copy)]
pub struct ServerAddresses {
    pub tcp_address: SocketAddrV6,
    pub udp_address: SocketAddrV6,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct LoginPacket {
    pub is_steam: bool,
    pub token: u64,
    /// The port the client will be using to send/recv packet over udp.
    pub client_udp_port: u16,
    /// Server version should match
    pub client_version: Version,
}
impl LoginPacket {
    pub const FIXED_SIZE: usize = 17;

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("could not serialize LoginPacket")
    }

    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        match bincode::deserialize::<Self>(&buffer) {
            Ok(result) => Some(result),
            Err(err) => {
                warn!("{} while trying to deserialize packet.", err);
                None
            }
        }
    }
}

#[test]
fn test_login_packet() {
    let og = LoginPacket {
        is_steam: false,
        token: 255,
        client_udp_port: 747,
        client_version: Version::CURRENT,
    };
    assert_eq!(og, LoginPacket::deserialize(&og.serialize()).unwrap());
    assert_eq!(og.serialize().len(), LoginPacket::FIXED_SIZE);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LoginResponsePacket {
    Accepted { client_id: ClientId },
    /// Client version does not match server version.
    WrongVersion { server_version: Version},
    /// Login without steam is not implemented.
    NotSteam,
    OtherError,
    /// Could not deserialize login response received from the server.
    DeserializeError,
}
impl LoginResponsePacket {
    pub const FIXED_SIZE: usize = 8;

    pub fn serialize(&self) -> Vec<u8> {
        match bincode::serialize(self) {
            Ok(v) => v,
            Err(err) => {
                warn!(
                    "{:?} while trying to serialize LoginResponsePacket. Sending empty packet...",
                    err
                );
                Vec::new()
            }
        }
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(buffer).unwrap_or(LoginResponsePacket::DeserializeError)
    }
}
#[test]
fn test_login_response_packet() {
    let og = LoginResponsePacket::Accepted {
        client_id: ClientId(1234),
    };
    assert_eq!(og, LoginResponsePacket::deserialize(&og.serialize()));
    assert_eq!(og.serialize().len(), LoginResponsePacket::FIXED_SIZE);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BattlescapeInput {
    /// Toggle firing selected weapon group.
    pub fire_toggle: bool,
    /// The angle of the capital ship wish direction.
    pub wish_dir: f32,
    /// The angle of the capital ship's selected weapons wish direction.
    pub aim_dir: f32,
    /// The absolute force of the capital ship wish direction.
    pub wish_dir_force: f32,
}
impl Default for BattlescapeInput {
    fn default() -> Self {
        Self {
            fire_toggle: false,
            wish_dir: 0.0,
            aim_dir: 0.0,
            wish_dir_force: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UdpClient {
    /// Could not deserialize packet.
    Invalid,
    Battlescape {
        wish_input: BattlescapeInput,
        /// A Battlescape command that has been received.
        acknowledge_command: u32,
    },
    Metascape {
        /// Where this client's currently controlled fleet wish to go.
        wish_position: Vec2,
    },
}
impl UdpClient {
    pub const MAX_SIZE: usize = 200;

    /// Serialize into a buffer ready to be sent over Udp.
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize::<Self>(&buffer).unwrap_or_default()
    }
}
impl Default for UdpClient {
    fn default() -> Self {
        Self::Invalid
    }
}

#[test]
fn test_udp_client() {
    let og = UdpClient::Battlescape {
        wish_input: BattlescapeInput {
            fire_toggle: true,
            wish_dir: 123.4,
            aim_dir: 777.7,
            wish_dir_force: 10.01,
        },
        acknowledge_command: 50,
    };
    println!("{:?}", &og);
    println!("{:?}", UdpClient::deserialize(&og.serialize()));
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetascapeStatePart  {
    pub tick: u32,
    /// This may only be a part of the whole state. This is the part number.
    pub part: u8,
    /// You need this entity order (from tcp) to make sense of this packet.
    pub entity_order_required: u8,
    /// Entities positions are relative to this position and not world origin.
    pub relative_position: Vec2,
    /// Sorted by entity id (see entity_order_required).
    /// What is the entity and their order is sent over tcp.
    /// TODO: Entity position is compressed from 8 bytes into 4 bytes.
    pub entities_position: Vec<Vec2>,
}
impl MetascapeStatePart {
    /// One UdpServer::Metascape packet will contain at most this amount of positions per packet.
    pub const NUM_ENTITIES_POSITION_MAX: usize = 100;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UdpServer {
    /// Could not deserialize packet.
    Invalid,
    Battlescape {
        battlescape_tick: u32,
        client_inputs: Vec<BattlescapeInput>,
    },
    MetascapeEntityPosition(MetascapeStatePart),
}
impl UdpServer {
    pub const MAX_SIZE: usize = 1200;

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        match bincode::deserialize::<Self>(&buffer) {
            Ok(result) => Some(result),
            Err(err) => {
                warn!("{} while trying to deserialize packet.", err);
                None
            }
        }
    }
}
impl Default for UdpServer {
    fn default() -> Self {
        Self::Invalid
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TcpPacket {
    EntityList {
        tick: u32,
        entity_order_id: u8,
        /// The order that the server will send entity info.
        /// These are the id of entity on the server.
        list: Vec<u32>,
    },
    FleetInfo {
        entity_id: u32,
        fleet_id: FleetId,
    }
}
impl TcpPacket {
    /// Tcp packet above this size will be ignored.
    pub const MAX_SIZE: usize = u16::MAX as usize;

    pub fn serialize(&self) -> Option<Vec<u8>> {
        match bincode::serialize(self) {
            Ok(v) => {
                if v.len() <= Self::MAX_SIZE {
                    Some(v)
                } else {
                    warn!("Tried to serialize a TcpPacket of {} which is above size limit of {}.", v.len(), Self::MAX_SIZE);
                    None
                }
            }
            Err(err) => {
                warn!("{} while trying to serialize TcpPacket.", err);
                None
            }
        }
    }

    pub fn serialized_size(&self) -> Option<usize> {
        match bincode::serialized_size(self) {
            Ok(num) => Some(num as usize),
            Err(err) => {
                warn!("{:?} while trying to get the serialized size of a TcpPacket.", err);
                None
            }
        }
    }

    pub fn serialize_into(&self, buf: &mut [u8]) -> bool {
        if let Err(err) = bincode::serialize_into(buf, self) {
            warn!("{:?} while trying to serialize TcpPacket into a provided buffer.", err);
            false
        } else {
            true
        }
    }

    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        match bincode::deserialize::<Self>(&buffer) {
            Ok(result) => Some(result),
            Err(err) => {
                debug!("{} while trying to deserialize TcpPacket.", err);
                None
            }
        }
    }
}
