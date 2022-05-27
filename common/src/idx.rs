use std::ops::{Index, IndexMut, AddAssign};
use utils::{Incrementable, acc::Filter};
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
impl AddAssign for ClientId {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl Incrementable for ClientId {
    fn one() -> Self {
        ClientId(0)
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

    pub fn is_client(self) -> bool {
        self.0 <= u64::from(u32::MAX)
    }
}
impl From<ClientId> for FleetId {
    fn from(client_id: ClientId) -> Self {
        Self(client_id.0 as u64)
    }
}
impl AddAssign for FleetId {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl Incrementable for FleetId {
    fn one() -> Self {
        Self(1)
    }
}

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u64);
impl AddAssign for FactionId {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl Incrementable for FactionId {
    fn one() -> Self {
        Self(1)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct SystemId(pub u32);
impl Index<SystemId> for Vec<System> {
    type Output = System;

    fn index(&self, index: SystemId) -> &Self::Output {
        &self[index.0 as usize]
    }
}
impl IndexMut<SystemId> for Vec<System> {
    fn index_mut(&mut self, index: SystemId) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}
impl AddAssign for SystemId {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl Incrementable for SystemId {
    fn one() -> Self {
        Self(1)
    }
}
impl Filter for SystemId {
    fn compare(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

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

/// TODO: What is that????
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InterceptionId(u32);
impl InterceptionId {
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }
}
