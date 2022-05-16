use std::ops::{Index, IndexMut};
use serde::{Deserialize, Serialize};
use crate::{
    ships::{ShipBase, WeaponBase},
    systems::System,
};

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
pub struct FactionId(pub u64);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SystemId(pub u16);
impl Index<SystemId> for Vec<System> {
    type Output = System;

    fn index(&self, index: SystemId) -> &Self::Output {
        &self[usize::from(index.0)]
    }
}
impl IndexMut<SystemId> for Vec<System> {
    fn index_mut(&mut self, index: SystemId) -> &mut Self::Output {
        &mut self[usize::from(index.0)]
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlanetId {
    pub system_id: SystemId,
    pub planets_offset: u8,
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

/// TODO: What is that????
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InterceptionId(u32);
impl InterceptionId {
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }
}
