use crate::idx::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CredentialChecker {
    Steam,
    Epic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginPacket {
    pub credential_checker: CredentialChecker,
    pub token: u64,
    /// Server/client version should match.
    pub client_version: String,
}
impl LoginPacket {
    pub const FIXED_SIZE: usize = 1000;

    pub fn serialize(&self) -> Vec<u8> {
        let mut v = bincode::serialize(self).expect("could not serialize LoginPacket");
        v.resize(Self::FIXED_SIZE, 0);
        v
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LoginResponsePacket {
    Accepted {
        client_id: ClientId,
    },
    /// Client version does not match server version.
    WrongVersion {
        server_version: String,
    },
    /// Selected server does not exist.
    UnknowServer,
    /// This is not implemented.
    NotImplemented,
    OtherError,
    /// Could not deserialize login response.
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
fn test_login_packet() {
    let og = LoginPacket {
        credential_checker: CredentialChecker::Steam,
        token: 255,
        client_version: crate::VERSION.to_string(),
    };
    assert_eq!(og, LoginPacket::deserialize(&og.serialize()).unwrap());
    assert_eq!(og.serialize().len(), LoginPacket::FIXED_SIZE);
}

#[test]
fn test_login_response_packet() {
    let og = LoginResponsePacket::Accepted {
        client_id: ClientId(1234),
    };
    assert_eq!(og, LoginResponsePacket::deserialize(&og.serialize()));
    assert_eq!(og.serialize().len(), LoginResponsePacket::FIXED_SIZE);
}
