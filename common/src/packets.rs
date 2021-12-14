use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use crate::idx::*;

#[derive(Debug, Clone, Copy)]
pub struct ServerAddresses {
    pub tcp_address: SocketAddr,
    pub udp_address: SocketAddr,
}

#[derive(Debug, Clone, Copy)]
pub enum PacketError {
    TooLarge,
    WrongSize,
    NoHeader,
    NoPayload,
    BincodeError,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct LoginPacket {
    pub is_steam: bool,
    pub token: u64,
    pub client_udp_port: u16,
}
impl LoginPacket {
    pub const FIXED_SIZE: usize = 11;

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("could not serialize LoginPacket")
    }

    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        match bincode::deserialize::<Self>(&buffer) {
            Ok(result) => Some(result),
            Err(err) => {
                debug!("{} while trying to deserialize LoginPacket.", err);
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
    };
    assert_eq!(og, LoginPacket::deserialize(&og.serialize()).unwrap());
    assert_eq!(og.serialize().len(), LoginPacket::FIXED_SIZE);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LoginResponsePacket {
    Accepted{
        client_id: ClientId
    },
    Error,
    DeserializeError,
}
impl LoginResponsePacket {
    pub const FIXED_SIZE: usize = 8;

    pub fn serialize(&self) -> Vec<u8> {
        match bincode::serialize(self) {
            Ok(v) => v,
            Err(err) => {
                warn!("{:?} while trying to serialize LoginResponsePacket. Sending empty packet...", err);
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
    let og = LoginResponsePacket::Accepted { client_id: ClientId(1234)};
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
    /// These packets are always the same size.
    pub const FIXED_SIZE: usize = 100;

    /// Serialize into a buffer ready to be sent over Udp.
    pub fn serialize(&self) -> Vec<u8> {
        let payload = bincode::serialize(self).unwrap();
        let mut v = Vec::with_capacity(Self::FIXED_SIZE);
        v.push(payload.len() as u8);
        v.extend_from_slice(&payload);
        v.resize(Self::FIXED_SIZE, 0);
        v
    }

    /// Deserialize from a buffer received from Udp.
    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        let size = match buffer.first() {
            Some(b) => *b as usize,
            None => {
                return None;
            }
        };

        if size <= 1 {
            return None;
        }

        if buffer.len() < size + 1 {
            return None;
        }

        match bincode::deserialize::<Self>(&buffer[1..size + 1]) {
            Ok(result) => Some(result),
            Err(_) => None,
        }
    }

    /// TODO: Serialize directly into a buffer.
    pub fn serialize_into(&self, mut _buf: &mut [u8]) {
        todo!()
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
    assert_eq!(og.serialize().len(), UdpClient::FIXED_SIZE);
    println!("{:?}", &og);
    println!("{:?}", UdpClient::deserialize(&og.serialize()).unwrap());
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UdpServer {
    Battlescape {
        battlescape_tick: u64,
        client_inputs: Vec<BattlescapeInput>,
    },
    Metascape {
        metascape_tick: u64,
        part: u8,
        /// Sorted by entity.
        /// What is the entity is sent over tcp.
        entities_position: Vec<Vec2>,
    },
}
impl UdpServer {
    // TODO: This should be 1200.
    /// Payload maximum size.
    pub const PAYLOAD_MAX_SIZE: usize = u8::MAX as usize;
    /// One UdpServer::Metascape packet will contain at most this amount of positions.
    pub const ENTITIES_POSITION_NUM_MAX: usize = 25;

    pub fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let payload = bincode::serialize(self).unwrap();

        // Check that we do not try to send a packet above max size.
        if payload.len() >= Self::PAYLOAD_MAX_SIZE {
            return Err(PacketError::TooLarge);
        }

        let mut v = Vec::with_capacity(Self::PAYLOAD_MAX_SIZE);
        v.push(payload.len() as u8);
        v.extend_from_slice(&payload);
        Ok(v)
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        // Get the size of the payload.
        let payload_size = match buffer.first() {
            Some(b) => *b as usize,
            None => {
                return Err(PacketError::NoHeader);
            }
        };

        if payload_size == 0 {
            return Err(PacketError::NoPayload);
        }

        if buffer.len() != payload_size + 1 {
            return Err(PacketError::WrongSize);
        }

        match bincode::deserialize::<Self>(&buffer[1..]) {
            Ok(result) => Ok(result),
            Err(_) => Err(PacketError::BincodeError),
        }
    }

    /// TODO: Serialize directly into a buffer.
    pub fn serialize_into(&self, mut _buf: &mut [u8]) {
        todo!()
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
    pub fn deserialize(buffer: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(buffer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TcpServer {
    EntityList {
        tick: u64,
        /// The order that the server will send entity info.
        list: Vec<ServerEntity>,
    },
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
    pub fn deserialize(buffer: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(buffer)
    }
}
