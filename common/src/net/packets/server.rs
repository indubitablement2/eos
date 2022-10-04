use super::Packet;
use crate::idx::*;
use bincode::Options;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use crate::command::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ServerPacket {
    /// Could not deserialize/serialize packet.
    #[default]
    Invalid,

    MetascapeTickCommands(MetascapeTickCommands),

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

/// Server send the commands for a tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetascapeTickCommands {
    /// The tick these commands apply to.
    pub tick: u64,
    /// The commands along with who sent them.
    pub cmds: Vec<TickCmd>,
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
        }
    }
}

/// The lenght of the connection queue before you.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ConnectionQueueLen {
    pub len: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginAccepted {
    pub client_id: ClientId,
    pub tick: u32,
    pub total_tick: u64,
}

/// Sent once after a client try to login.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoginResponse {
    Accepted(LoginAccepted),
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
