use crate::idx::ClientId;
use serde::{Deserialize, Serialize};

/// Packet originating from client. Meant for things outside a sector ex: trade, quest channel message.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClientLoginPacket {
    /// If deserialization was successful. You can set this to anything when making a new packet.
    pub deserialize_result: bool,
    /// Use eos_common::const_var::APP_VERSION
    pub app_version: u32,
    pub steam_id: ClientId,
    /// Convert the ticket from GetAuthSessionTicket from binary to hex into an appropriately sized byte character array.
    pub ticket: String,
}

impl crate::packet_common::Packetable for ClientLoginPacket {
    fn serialize(&self) -> (Vec<u8>, u8) {
        (bincode::serialize(self).unwrap_or_default(), 10)
    }

    fn deserialize(bytes: &[u8]) -> Self
    where
        Self: Sized,
    {
        bincode::deserialize(bytes).unwrap_or_default()
    }
}

impl Default for ClientLoginPacket {
    fn default() -> Self {
        ClientLoginPacket {
            deserialize_result: false,
            app_version: 0,
            steam_id: ClientId(0),
            ticket: String::new(),
        }
    }
}
