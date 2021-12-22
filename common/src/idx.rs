use serde::{Deserialize, Serialize};

/// 0 is reserved and means invalid.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClientId(pub u32);
impl ClientId {
    /// Return if this is a valid ClientId, id != 0.
    pub fn is_valid(self) -> bool {
        self.0 != 0
    }
}
impl From<FleetId> for ClientId {
    fn from(fleet_id: FleetId) -> Self {
        if fleet_id.0 > u32::MAX as u64 {
            Self(0)
        } else {
            Self(fleet_id.0 as u32)
        }
    }
}

/// Never recycled.
/// First 2^32 - 1 idx are reserved for clients.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FleetId(pub u64);
impl From<ClientId> for FleetId {
    fn from(client_id: ClientId) -> Self {
        Self(client_id.0 as u64)
    }
}
#[test]
fn fleet_client_id() {
    let client_id = ClientId(123);
    let to_fleet_id = FleetId::from(client_id);
    println!("client: {:?}", to_fleet_id);

    let ai_fleet_id = FleetId(u32::MAX as u64 + 1);
    println!("ai: {:?}", ai_fleet_id);
    let ai_client_id = ClientId::from(ai_fleet_id);
    println!("ai: {:?}", ai_client_id);
    assert!(!ai_client_id.is_valid());
}

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u32);

pub struct SystemId(pub u32);

pub struct CelestialBodyId {
    pub system_id: SystemId,
    pub body_index: u8,
}
