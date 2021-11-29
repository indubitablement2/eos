use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr};

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
    BincodeError
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct LoginPacket {
    pub is_steam: bool,
    pub token: u64,
    pub udp_address: SocketAddr,
}
impl LoginPacket {
    pub const FIXED_SIZE: usize = 50;

    pub fn serialize(&self) -> Vec<u8> {
        let payload = bincode::serialize(self).unwrap();
        let mut v = Vec::with_capacity(Self::FIXED_SIZE);
        v.push(payload.len() as u8);
        v.extend_from_slice(&payload);
        v.resize(Self::FIXED_SIZE, 0);
        v
    }

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
}

#[test]
fn test_login_packet() {
    let og = LoginPacket {
        is_steam: false,
        token: 255,
        udp_address: SocketAddr::new(
            std::net::IpAddr::V6(std::net::Ipv6Addr::new(123, 444, 555, 7211, 1123, 34509, 111, 953)),
            747,
        ),
    };
    assert_eq!(og, LoginPacket::deserialize(&og.serialize()).unwrap());
    assert_eq!(og.serialize().len(), LoginPacket::FIXED_SIZE);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoginResponsePacket {
    Accepted,
    Error,
}
impl LoginResponsePacket {
    pub const FIXED_SIZE: usize = 1;

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            LoginResponsePacket::Accepted => vec![0],
            LoginResponsePacket::Error => vec![255],
        }
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        match buffer[0] {
            0 => Self::Accepted,
            _ => Self::Error,
        }
    }
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
        client_inputs: Vec<BattlescapeInput>,
        tick: u64,
    },
    Metascape {
        fleets_position: Vec<Vec2>,
        tick: u64,
    },
}
impl UdpServer {
    /// These packet have a maximum size.
    pub const MAX_SIZE: usize = u8::MAX as usize;

    pub fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let payload = bincode::serialize(self).unwrap();

        // Check that we do not try to send a packet above max size.
        if payload.len() >= Self::MAX_SIZE {
            return Err(PacketError::TooLarge);
        }

        let mut v = Vec::with_capacity(Self::MAX_SIZE);
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
pub enum TcpServer {}
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
