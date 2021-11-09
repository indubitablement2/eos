use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct LoginPacket {
    pub is_steam: bool,
    pub token: u64,
    pub udp_address: SocketAddr,
}
impl LoginPacket {
    pub const FIXED_SIZE: usize = 40;

    pub fn serialize(&self) -> Vec<u8> {
        let payload = bincode::serialize(self).unwrap();
        let mut v = Vec::with_capacity(payload.len() + 1);
        v.push(payload.len() as u8);
        v.extend_from_slice(&payload);
        v
    }

    /// Deserialize from a buffer received from Udp.
    pub fn deserialize(buffer: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        let size = buffer[0] as usize;
        bincode::deserialize(&buffer[1..size + 1])
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
        wish_position: Vector2<f32>,
    },
}
impl UdpClient {
    /// TODO: What is the size of a UdpClient?
    pub const FIXED_SIZE: usize = 50;

    /// Serialize into a buffer ready to be sent over Udp.
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    /// Deserialize from a buffer received from Udp.
    pub fn deserialize(buffer: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(buffer)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UdpServer {
    Battlescape {
        client_inputs: Vec<BattlescapeInput>,
        tick: u32,
    },
    Metascape {
        fleets_position: Vec<Vector2<f32>>,
    },
}
impl UdpServer {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(buffer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TcpClient {}
impl TcpClient {
    pub const MAX_SIZE: usize = 131072;

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
