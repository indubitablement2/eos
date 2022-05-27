use crate::{idx::*, orbit::Orbit};
use utils::compressed_vec2::*;
use battlescape::{commands::BattlescapeCommand, player_inputs::PlayerInput};
use bincode::Options;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BattlescapeCommands {
    pub commands: Vec<(u32, Vec<BattlescapeCommand>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesState {
    pub tick: u32,
    pub client_entity_position: Vec2,
    /// Entity's id and position compressed and relative to client's position.
    pub relative_entities_position: Vec<(u16, CVec2)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInfo {
    pub fleet_id: FleetId,
    /// The ships composing the fleet.
    pub composition: Vec<ShipBaseId>,
}

// TODO: Separate fleet from cargo (only 2 type of entity with very different behavior).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityInfoType {
    Fleet(FleetInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub info_type: EntityInfoType,
    pub name: String,
    /// If this entity follow an orbit, its state will not be sent.
    pub orbit: Option<Orbit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesInfo {
    /// This is useful with orbit.
    /// Any state before this tick can be discarded and apply the orbit instead.
    /// Any state after this tick will remove the orbit.
    pub tick: u32,
    /// The client's (fleet) info, if it has changed.
    pub client_info: Option<EntityInfo>,
    pub infos: Vec<(u16, EntityInfo)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetsPosition {
    pub tick: u32,
    /// Position in world space.
    pub client_position: Vec2,
    /// Detected fleets position compressed and relative to client's position.
    /// Ordered by `FleetId`. 
    /// See: `DetectedFleetsInfos`.
    pub relative_fleets_position: Vec<CVec2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInfos {
    pub fleet_id: FleetId,
    pub name: String,
    /// If this entity follow an orbit, its state will not be sent.
    pub orbit: Option<Orbit>,
    /// The ships composing the fleet.
    pub composition: Vec<ShipBaseId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFleetsInfos {
    /// This is useful with orbit.
    /// Any state before this tick can be discarded and apply the orbit instead.
    /// Any state after this tick will remove the orbit.
    pub tick: u32,
    /// May include the client's
    pub infos: Vec<FleetInfos>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesRemove {
    pub tick: u32,
    /// Free these entities. Their idx will be reused in the future.
    pub to_remove: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectedReasonEnum {
    /// Server received an invalid packet.
    InvalidPacket,
    /// Someone else connected on the same account.
    ConnectionFromOther,
    /// The server has encountered a fatal error through no fault of the client.
    ServerError,
}
impl Display for DisconnectedReasonEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisconnectedReasonEnum::InvalidPacket => write!(f, "Server received an invalid packet."),
            DisconnectedReasonEnum::ConnectionFromOther => write!(f, "Someone else connected on the same account."),
            DisconnectedReasonEnum::ServerError => write!(
                f,
                "The server has encountered a fatal error through no fault of the client."
            ),
        }
    }
}


/// The lenght of the connection queue before you.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ConnectionQueueLen {
    pub len: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ServerPacket {
    /// Could not deserialize/serialize packet.
    #[default]
    Invalid,

    /// Server send this for every commands that are not acknowledged by the client.
    BattlescapeCommands(BattlescapeCommands),
    /// Server send some entities's position.
    EntitiesState(EntitiesState), // TODO: Delete
    /// Server send some entities's infos.
    EntitiesInfo(EntitiesInfo), // TODO: Delete
    /// Server send entities that will not be updated anymore and should be removed.
    EntitiesRemove(EntitiesRemove), // TODO: Delete
    /// Server send the reason why it disconnected the client.
    DisconnectedReason(DisconnectedReasonEnum),
    /// Lenght of the queue before the client.
    ConnectionQueueLen(ConnectionQueueLen),
    /// Infoes about currently detected fleets
    DetectedFleetsInfos(DetectedFleetsInfos),
    /// Position of the client's fleet and detected fleets.
    FleetsPosition(FleetsPosition),
}
impl ServerPacket {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new().serialize(self).unwrap_or_default()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::DefaultOptions::new().deserialize(buffer).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ClientPacket {
    /// Could not deserialize/serialize packet.
    #[default]
    Invalid,
    /// Client send this when he wants his fleet to move to a position.
    MetascapeWishPos { wish_pos: Vec2, movement_multiplier: f32 }, // TODO: Add sequence #
    BattlescapeInput {
        wish_input: PlayerInput,
        /// The last Battlescape commands the client acknowledge to have received.
        /// All commands before are implicitely acknowledged and will not be resent be the server.
        last_acknowledge_command: u32,
    },
}
impl ClientPacket {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new().serialize(self).unwrap_or_default()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::DefaultOptions::new().deserialize(buffer).unwrap_or_default()
    }
}
