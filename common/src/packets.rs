use crate::{idx::*, position::Position, Version};
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct LoginPacket {
    pub is_steam: bool,
    pub token: u64,
    /// The port the client will be using to send/recv packet over udp.
    pub client_udp_port: u16,
    /// Server/client version should match.
    pub client_version: Version,
}
impl LoginPacket {
    pub const FIXED_SIZE: usize = 17;

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("could not serialize LoginPacket")
    }

    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        match bincode::deserialize::<Self>(buffer) {
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
    Accepted {
        client_id: ClientId,
    },
    /// Client version does not match server version.
    WrongVersion {
        server_version: Version,
    },
    /// Login without steam is not implemented.
    NotSteam,
    /// Provided udp port is not valid.
    BadUDPPort {
        provided_port: u16,
    },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    entity_id: u32,
    position: Position,
    velocity: Vec2,
    wish_position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    entity_id: u32,
    /// TODO: This should be computed.
    acceleration: Vec2,
    fleet_id: Option<FleetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeCommand {
    tick: u16,
    clients_inputs: Vec<BattlescapeInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectedReasonEnum {
    /// Server received an invalid packet.
    InvalidPacket,
    /// Someone else connected on the same account.
    ConnectionFromOther
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    /// Could not deserialize/serialize packet.
    Invalid,
    /// Server send this for every commands that are not acknowledged by the client.
    BattlescapeCommands { commands: Vec<BattlescapeCommand> },
    /// Server send some entities's position.
    EntitiesState {
        tick: u32,
        client_entity_state: EntityState,
        /// Compressed and relative to client's world position.
        ///
        /// TODO: Compressed
        relative_entities_states: Vec<EntityState>,
    },
    /// Server send some entities's infos.
    EntityInfo(Vec<EntityInfo>),
    DisconnectedReason(DisconnectedReasonEnum),

    Message {
        /// Invalid ClientId means the origin is the server.
        origin: ClientId,
        content: String,
    },

    /// Client send this when he wants his fleet to move to a position.
    MetascapeWishPos {
        wish_pos: Vec2,
    },
    /// Client send his battlescape inputs and last acknowledged commands.
    BattlescapeInput {
        wish_input: BattlescapeInput,
        /// The last Battlescape command the client acknowledge to have received.
        /// All commands before are implicitly acknowledged and will not be resent be the server.
        last_acknowledge_command: u16,
    },
}
impl Packet {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize::<Self>(buffer).unwrap_or_default()
    }
}
impl Default for Packet {
    fn default() -> Self {
        Self::Invalid
    }
}
