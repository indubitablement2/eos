use super::Packet;
use crate::{idx::*, net::auth::CredentialChecker};
use bincode::Options;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ClientPacket {
    /// Could not deserialize/serialize packet.
    #[default]
    Invalid,

    LoginPacket(LoginPacket),
    // ClientInputs {
    //     last_metascape_state_ack: u32,
    //     inputs: ClientInputsType,
    // },
}
impl Packet for ClientPacket {
    fn serialize(&self) -> Vec<u8> {
        bincode::DefaultOptions::new().serialize(self).unwrap_or_default()
    }

    fn deserialize(buffer: &[u8]) -> Self {
        bincode::DefaultOptions::new().deserialize(buffer).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginPacket {
    pub credential_checker: CredentialChecker,
    /// If the auth is successful, you will receive udp packets to this address.
    pub requested_udp_addr: SocketAddr,
}
