use super::Packet;
use crate::{fleet::FleetComposition, idx::*};
use bincode::Options;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utils::compressed_vec2::CVec2;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ServerPacket {
    /// Could not deserialize/serialize packet.
    #[default]
    Invalid,

    MetascapeState(MetascapeState),
    FleetsInfos(FleetsInfos),
    FactionsInfo(FactionsInfo),

    DisconnectedReason(DisconnectedReason),
    ConnectionQueueLen(ConnectionQueueLen),
    LoginResponse(LoginResponse),
}
impl Packet for ServerPacket {
    fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new().serialize(self).unwrap_or_default()
    }

    fn deserialize(buffer: &[u8]) -> Self {
        bincode::DefaultOptions::new().deserialize(buffer).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetascapeStateOrderChange {
    pub change_tick: u32,
    /// Fleet added this period.
    pub order_add: Vec<FleetId>,
    /// Fleet removed this period.
    pub order_remove: Vec<FleetId>,
}
impl MetascapeStateOrderChange {
    pub fn size(&self) -> usize {
        4 + 8 + 8 + self.order_add.len() * 8 + self.order_remove.len() * 8
    }
}

/// Position of the client's fleet and detected fleets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetascapeState {
    pub tick: u32,
    /// Client has no fleet.
    pub ghost: bool,
    pub order_checksum: u32,
    /// Order changes that were not acknowleged.
    pub non_ack_change: Vec<MetascapeStateOrderChange>,
    /// Position in world space that fleets position are relative to.
    /// Also the position of the client's fleet if it exist.
    pub origin: Vec2,
    /// Detected fleets position compressed and relative to `origin`.
    pub relative_fleets_position: Vec<CVec2<512>>,
    /// Fleet that are sent in the current order.
    pub sent_order_start: u16,
}
impl MetascapeState {
    pub const BASE_SIZE: usize = 4 + 1 + 4 + 8 + 8 + 8 + 2;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInfos {
    pub fleet_id: FleetId,
    pub name: String,
    pub fleet_composition: FleetComposition,
}

/// Infos about currently detected fleets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetsInfos {
    pub tick: u32,
    /// New fleets along with its full infos.
    pub new_fleets: Vec<FleetInfos>,
    /// `FleetComposition`s that changed.
    pub compositions_changed: Vec<(FleetId, FleetComposition)>,
}

/// TODO
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactionsInfo {
    // pub tick: u32,
    // /// New fleets along with its full infos.
    // pub new_fleets: Vec<FleetInfos>,
    // /// `FleetComposition`s that changed.
    // pub compositions_changed: Vec<(FleetId, FleetComposition)>,
}

/// Server sent the reason why it disconnected the client.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DisconnectedReason {
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
    /// Client/server data is out of sync.
    DataOutOfSync,
    /// Client/server state order is out of sync. 
    /// Detected on the client side with a checksum.
    /// This is a bug.
    OrderOutOfSync,
}
impl Display for DisconnectedReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisconnectedReason::InvalidPacket => write!(f, "Server received an invalid packet."),
            DisconnectedReason::ConnectionFromOther => write!(f, "Someone else connected on the same account."),
            DisconnectedReason::ServerError => write!(
                f,
                "The server has encountered a fatal error through no fault of the client."
            ),
            DisconnectedReason::DeserializeError => write!(f, "Client could not deserialize a server packet."),
            DisconnectedReason::LostConnection => write!(f, "Client lost connection to the server."),
            DisconnectedReason::DataOutOfSync => write!(f, "Client/server data is out of sync."),
            DisconnectedReason::OrderOutOfSync => write!(f, "Client/server state order is out of sync. This is a bug."),
        }
    }
}

/// The lenght of the connection queue before you.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ConnectionQueueLen {
    pub len: u32,
}

/// Sent once after a client try to login.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoginResponse {
    Accepted {
        client_id: ClientId,
    },
    /// Failed to authenticate.
    BadCredential,
    /// First received packet should be a `LoginPacket`.
    FirstPacketNotLogin,
    /// Did not receive login packet in time.
    LoginTimeOut,
    ServerFull,
    /// The server has no socket that can handle the client's requested udp address's ip protocole.
    NoValidSocket,
}
