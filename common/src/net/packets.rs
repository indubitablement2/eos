use crate::{fleet::FleetComposition, idx::*, orbit::Orbit};
use bincode::Options;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utils::compressed_vec2::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetsPosition {
    pub tick: u32,
    /// Position in world space fleets position are relative to.
    pub origin: Vec2,
    /// Detected fleets `small_id` and position compressed and relative to client's position.
    /// See: `DetectedFleetsInfos`.
    pub relative_fleets_position: Vec<(u16, CVec2<512>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInfos {
    pub fleet_id: FleetId,
    /// Client's fleet is always 0.
    pub small_id: u16,
    pub name: String,
    /// If this entity follow an orbit, its state will not be sent.
    pub orbit: Option<Orbit>,
    pub fleet_composition: FleetComposition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetsInfos {
    /// This is useful with orbit.
    /// Any state before this tick can be discarded and apply the orbit instead.
    /// Any state after this tick will remove the orbit.
    pub tick: u32,
    /// May include the client's fleet.
    pub infos: Vec<FleetInfos>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetsForget {
    pub tick: u32,
    /// Forget these fleets. Their small_idx will be reused in the future.
    pub to_forget: Vec<u16>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DisconnectedReasonEnum {
    /// Server received an invalid packet.
    InvalidPacket,
    /// Someone else connected on the same account.
    ConnectionFromOther,
    /// The server has encountered a fatal error through no fault of the client.
    ServerError,

    /// Client could not deserialize a server packet.
    DeserializeError,
    /// Client lost connection to the server.
    LostConnection,
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
            DisconnectedReasonEnum::DeserializeError => write!(f, "Client could not deserialize a server packet."),
            DisconnectedReasonEnum::LostConnection => write!(f, "Client lost connection to the server."),
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

    // /// Server send this for every commands that are not acknowledged by the client.
    // BattlescapeCommands(BattlescapeCommands),
    /// Server send the reason why it disconnected the client.
    DisconnectedReason(DisconnectedReasonEnum),
    /// Lenght of the queue before the client.
    ConnectionQueueLen(ConnectionQueueLen),
    /// Infos about currently detected fleets
    FleetsInfos(FleetsInfos),
    /// Position of the client's fleet and detected fleets.
    FleetsPosition(FleetsPosition),
    FleetsForget(FleetsForget),
    /// Fleets owned by the client.
    OwnedFleets(Vec<FleetId>),
    /// Currently controlled fleet.
    FleetControl(Option<FleetId>),
}
impl ServerPacket {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new().serialize(self).unwrap_or_default()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::DefaultOptions::new().deserialize(buffer).unwrap_or_default()
    }
}

/// Client can only send packet through tcp.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ClientPacket {
    /// Could not deserialize/serialize packet.
    #[default]
    Invalid,
    /// Client send this when he wants his fleet to move to a position.
    MetascapeWishPos { wish_pos: Vec2, movement_multiplier: f32 },
    /// Client ask to create a starting fleet to take control of it.
    CreateStartingFleet {
        starting_fleet_id: StartingFleetId,
        /// Where the fleet should spawn.
        location: PlanetId,
    },
    /// Client request to take control of one of his fleet.
    ControlOwnedFleet {
        fleet_id: Option<FleetId>,
    },
    // BattlescapeInput {
    //     wish_input: PlayerInput,
    //     /// The last Battlescape commands the client acknowledge to have received.
    //     /// All commands before are implicitely acknowledged and will not be resent be the server.
    //     last_acknowledge_command: u32,
    // },
}
impl ClientPacket {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new().serialize(self).unwrap_or_default()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::DefaultOptions::new().deserialize(buffer).unwrap_or_default()
    }
}
