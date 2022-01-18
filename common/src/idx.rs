use serde::{Deserialize, Serialize};

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClientId(pub u32);
impl ClientId {
    pub fn to_fleet_id(self) -> FleetId {
        FleetId(self.0.into())
    }
}

/// Never recycled.
/// First 2^32 - 1 idx are reserved for clients.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FleetId(pub u64);
impl FleetId {
    pub fn to_client_id(self) -> Option<ClientId> {
        if let Ok(id) = u32::try_from(self.0) {
            Some(ClientId(id))
        } else {
            None
        }
    }
}

impl From<ClientId> for FleetId {
    fn from(client_id: ClientId) -> Self {
        Self(client_id.0 as u64)
    }
}
impl FleetId {
    pub fn is_client(self) -> bool {
        self.0 <= u64::from(u32::MAX)
    }
}

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u32);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SystemId(pub u32);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlanetId {
    pub system_id: SystemId,
    pub planets_offset: u8,
}
