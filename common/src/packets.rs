use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
    /// Serialize into a buffer ready to be sent over Udp.
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    /// Deserialize from a buffer received from Udp.
    pub fn deserialize(buffer: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(buffer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
