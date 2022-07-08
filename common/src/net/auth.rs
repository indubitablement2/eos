use crate::idx::ClientId;
use serde::{Deserialize, Serialize};

/// Something to verify who this person is.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CredentialChecker {
    Local { client_id: ClientId },
    Steam(u64),
}
impl CredentialChecker {
    /// Verify who this is.
    pub async fn check(&self) -> Option<Auth> {
        match self {
            CredentialChecker::Local { client_id: _ } => None,
            CredentialChecker::Steam(_) => todo!(),
        }
    }
}

/// A connection was identified as this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Auth {
    /// Only created for offline mode.
    Local(ClientId),
    SteamId(u64),
}
