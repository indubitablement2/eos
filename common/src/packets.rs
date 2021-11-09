use std::net::SocketAddr;

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPacket {
    pub is_steam: bool,
    pub token: u64,
    pub udp_address: SocketAddr,
}
impl LoginPacket {
    /// TODO: What is the size of a SocketAddr?
    pub const FIXED_SIZE: usize = 100;

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    /// Deserialize from a buffer received from Udp.
    pub fn deserialize(buffer: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(buffer)
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
