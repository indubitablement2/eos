use crate::idx::*;
use crate::location::*;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Packet originating from client meant for a single sector.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientLocalPacket {
    Invalid,
    /// TODO: Client send where he would like one of his fleet to move.
    ClientFleetWishLocation {
        fleet_id: FleetId,
        location: Location,
    },
    /// Send a chat message.
    Broadcast {
        message: String,
    },
}

/// Packet originating from client. Meant for things outside a sector ex: trade, quest channel message.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientGlobalPacket {
    Invalid,
    /// Send a chat message.
    Broadcast {
        message: String,
    },
}

/// Packet originating from server or for client login.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OtherPacket {
    Invalid,

    /// Server respond to client Login.
    LoginResult {
        success: bool,
    },
    /// Packet originating from client. Used for login.
    ClientLogin {
        /// Use eos_common::const_var::APP_VERSION
        app_version: u32,
        steam_id: ClientId,
        /// Convert the ticket from GetAuthSessionTicket from binary to hex into an appropriately sized byte character array.
        ticket: String,
    },

    /// TODO: Server send position, velocity and wish position of fleets nearby.
    FleetsPositionVelWish {
        id: Vec<FleetId>,
        position: Vec<Vec2>,
        velocity: Vec<Vec2>,
        wish_position: Vec<Vec2>,
    },
    /// Send a chat message.
    Broadcast {
        importance: u8,
        message: String,
    },
}

/// Used to identify the type of packet. U8::Max is used as *lack of packet or none*.
pub trait PacketId {
    const ID: u8;
}

impl PacketId for ClientLocalPacket {
    const ID: u8 = 0;
}

impl PacketId for ClientGlobalPacket {
    const ID: u8 = 1;
}

impl PacketId for OtherPacket {
    const ID: u8 = 2;
}

pub trait Packetable {
    /// Serialize into a Vec<u8> and a byte to be sent to a Connection.
    fn serialize(&self) -> (Vec<u8>, u8);
    /// Deserialize some bytes into a Packet. Return Default if any error occur.
    fn deserialize(packet: &[u8]) -> Self
    where
        Self: Sized;
}

impl Packetable for ClientLocalPacket {
    fn serialize(&self) -> (Vec<u8>, u8) {
        (bincode::serialize(self).unwrap_or_default(), 0)
    }

    fn deserialize(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        bincode::deserialize(bytes).unwrap_or_default()
    }
}

impl Packetable for ClientGlobalPacket {
    fn serialize(&self) -> (Vec<u8>, u8) {
        (bincode::serialize(self).unwrap_or_default(), 1)
    }

    fn deserialize(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        bincode::deserialize(bytes).unwrap_or_default()
    }
}

impl Packetable for OtherPacket {
    fn serialize(&self) -> (Vec<u8>, u8) {
        (bincode::serialize(self).unwrap_or_default(), 2)
    }

    fn deserialize(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        bincode::deserialize(bytes).unwrap_or_default()
    }
}

impl Default for ClientLocalPacket {
    fn default() -> Self {
        Self::Invalid
    }
}

impl Default for ClientGlobalPacket {
    fn default() -> Self {
        Self::Invalid
    }
}

impl Default for OtherPacket {
    fn default() -> Self {
        Self::Invalid
    }
}
