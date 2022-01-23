use std::ops::{Index, IndexMut};

use ahash::AHashSet;
use serde::{Deserialize, Serialize};

use crate::{factions::*, systems::System};

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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u8);
impl FactionId {
    pub const MAX: u8 = u32::BITS as u8 - 1;

    pub fn to_bit_flag(self) -> u32 {
        1 << self.0
    }
}
impl Index<FactionId> for [Faction; Factions::MAX_FACTIONS] {
    type Output = Faction;

    fn index(&self, index: FactionId) -> &Self::Output {
        &self[index.0 as usize]
    }
}
impl IndexMut<FactionId> for [Faction; Factions::MAX_FACTIONS] {
    fn index_mut(&mut self, index: FactionId) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}
impl Index<FactionId> for [AHashSet<PlanetId>; Factions::MAX_FACTIONS] {
    type Output = AHashSet<PlanetId>;

    fn index(&self, index: FactionId) -> &Self::Output {
        &self[index.0 as usize]
    }
}
impl IndexMut<FactionId> for [AHashSet<PlanetId>; Factions::MAX_FACTIONS] {
    fn index_mut(&mut self, index: FactionId) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}

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
