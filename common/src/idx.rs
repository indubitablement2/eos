use crate::data::{ship::*, weapon::*};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::ops::Index;

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ClientId(pub u32);
impl ClientId {
    pub fn to_fleet_id(self) -> FleetId {
        FleetId(self.0.into())
    }
}

/// Never recycled.
/// First `2^32 - 1` id are reserved for client.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Pod, Zeroable, Default)]
#[repr(transparent)]
pub struct FleetId(pub u64);
impl FleetId {
    pub fn to_client_id(self) -> Option<ClientId> {
        u32::try_from(self.0).ok().map(|id| ClientId(id))
    }
}
impl From<ClientId> for FleetId {
    fn from(client_id: ClientId) -> Self {
        Self(client_id.0 as u64)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct FactionId(u8);
impl FactionId {
    pub fn new(mut id: u8) -> Self {
        if id >= 64 {
            log::warn!("Tried to create a faction id with id {}. Setting id to 0...", id);
            id = 0
        }
        Self(id)
    }

    pub fn id(&self) -> u8 {
        self.0
    }

    pub fn neutral(&self) -> bool {
        self.0 == 0
    }

    pub fn mask(&self) -> u64 {
        1 << self.0
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct SystemId(pub u32);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlanetId {
    pub system_id: SystemId,
    pub planets_offset: u32,
}

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BattlescapeId(u64);
impl BattlescapeId {
    pub fn from_raw(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShipBaseId(u32);
impl ShipBaseId {
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }
}
impl Index<ShipBaseId> for Vec<ShipBase> {
    type Output = ShipBase;

    fn index(&self, index: ShipBaseId) -> &Self::Output {
        &self[index.0 as usize]
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WeaponBaseId(u32);
impl WeaponBaseId {
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }
}
impl Index<WeaponBaseId> for Vec<WeaponBase> {
    type Output = WeaponBase;

    fn index(&self, index: WeaponBaseId) -> &Self::Output {
        &self[index.0 as usize]
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StartingFleetId(u32);
impl StartingFleetId {
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }

    pub fn id(self) -> u32 {
        self.0
    }
}
