use super::Packet;
use crate::{idx::*, net::auth::CredentialChecker};
use bincode::Options;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ClientPacket {
    /// Could not deserialize/serialize packet.
    #[default]
    Invalid,

    LoginPacket(LoginPacket),

    ClientInputs {
        last_metascape_state_ack: u32,
        inputs: ClientInputsType,
    },
    /// Client ask to create a starting fleet to take control of it.
    CreateStartingFleet {
        starting_fleet_id: StartingFleetId,
        /// Where the fleet should spawn.
        location: PlanetId,
    },
}
impl Packet for ClientPacket {
    fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new().serialize(self).unwrap_or_default()
    }

    fn deserialize(buffer: &[u8]) -> Self {
        bincode::DefaultOptions::new().deserialize(buffer).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum ClientInputsType {
    #[default]
    None,
    Metascape {
        wish_pos: Vec2,
        movement_multiplier: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginPacket {
    pub credential_checker: CredentialChecker,
    /// If the auth is successful, you will receive udp packets to this address.
    pub requested_udp_addr: SocketAddr,
}
