use crate::data::{ship::*, weapon::*};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::ops::Index;

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ClientId(pub u32);

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Pod, Zeroable, Default)]
#[repr(transparent)]
pub struct FleetId(pub u64);

/// 0 is reserved for the neutral faction.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct FactionId(pub u32);
impl FactionId {
    pub const NEUTRAL: Self = Self(0);

    pub fn is_neutral(self) -> bool {
        self == Self::NEUTRAL
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
