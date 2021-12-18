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
    WrongVersion,
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
    println!("{:?}", UdpClient::deserialize(&og.serialize()).unwrap());
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

#[derive(Debug, Serialize, Deserialize)]
pub enum TcpClient {}
impl TcpClient {
    pub const MAX_SIZE: usize = 65536;

    /// Adds a 32bits header representing payload size.
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = bincode::serialize(self).unwrap();
        let mut v = (payload.len() as u32).to_be_bytes().to_vec();
        v.append(&mut payload);
        v
    }

    /// Expect no header.
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

#[derive(Debug, Serialize, Deserialize)]
pub enum TcpServer {
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
impl TcpServer {
    /// Adds a 32bits header representing payload size.
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = bincode::serialize(self).unwrap();
        let mut v = (payload.len() as u32).to_be_bytes().to_vec();
        v.append(&mut payload);
        v
    }

    /// Expect no header.
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
