#![allow(unused)]

use super::*;
use entity_data_id_err::*;
use simulation::entity::EntityData;
use std::{
    num::{NonZeroU32, NonZeroU64},
    ops::Deref,
};

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "u32")]
#[serde(into = "u32")]
pub struct EntityDataId(pub &'static EntityData);
impl Default for EntityDataId {
    fn default() -> Self {
        Self(data().entities.first().unwrap())
    }
}
impl Deref for EntityDataId {
    type Target = EntityData;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
pub mod entity_data_id_err {
    pub struct TryFromEntityDataIdError(pub u32);
    impl std::fmt::Display for TryFromEntityDataIdError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Invalid entity data id: {} out of bound", self.0)
        }
    }
}
impl TryFrom<u32> for EntityDataId {
    type Error = TryFromEntityDataIdError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        data()
            .entities
            .get(0 as usize)
            .map(Self)
            .ok_or(TryFromEntityDataIdError(0))
    }
}
impl From<EntityDataId> for u32 {
    fn from(ptr: EntityDataId) -> Self {
        ptr.id
    }
}
impl std::fmt::Debug for EntityDataId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

const ENTITY_ID_START: u64 = 1u64 << 63;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(NonZeroU64);
impl EntityId {
    pub fn from_u64(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Self)
    }

    pub fn as_u64(self) -> u64 {
        self.0.get()
    }

    pub fn next(&mut self) -> Self {
        let current = *self;
        self.0 = self.0.checked_add(1).unwrap();
        current
    }

    pub fn to_ship_id(self) -> Option<ShipId> {
        if self.0.get() < ENTITY_ID_START {
            Some(ShipId(self.0))
        } else {
            None
        }
    }
}
impl Default for EntityId {
    fn default() -> Self {
        Self::from_u64(ENTITY_ID_START).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ShipId(NonZeroU64);
impl ShipId {
    pub fn from_u64(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Self)
    }

    pub fn as_u64(self) -> u64 {
        self.0.get()
    }

    pub fn next(&mut self) -> Self {
        let current = *self;
        self.0 = self.0.checked_add(1).unwrap();
        current
    }

    pub fn to_entity_id(self) -> EntityId {
        EntityId(self.0)
    }
}
impl Default for ShipId {
    fn default() -> Self {
        Self::from_u64(1).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SimulationId(NonZeroU64);
impl SimulationId {
    pub fn from_u64(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Self)
    }

    pub fn as_u64(self) -> u64 {
        self.0.get()
    }

    pub fn next(&mut self) -> Self {
        let current = *self;
        self.0 = self.0.checked_add(1).unwrap();
        current
    }
}
impl Default for SimulationId {
    fn default() -> Self {
        Self::from_u64(1).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(NonZeroU64);
impl ClientId {
    pub fn from_u64(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Self)
    }

    pub fn as_u64(self) -> u64 {
        self.0.get()
    }

    pub fn next(&mut self) -> Self {
        let current = *self;
        self.0 = self.0.checked_add(1).unwrap();
        current
    }
}
impl Default for ClientId {
    fn default() -> Self {
        Self::from_u64(1).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstanceId(NonZeroU32);
impl InstanceId {
    pub fn from_u32(id: u32) -> Option<Self> {
        NonZeroU32::new(id).map(Self)
    }

    pub fn as_uu32(self) -> u32 {
        self.0.get()
    }

    pub fn next(&mut self) -> Self {
        let current = *self;
        self.0 = self.0.checked_add(1).unwrap();
        current
    }
}
impl Default for InstanceId {
    fn default() -> Self {
        Self::from_u32(1).unwrap()
    }
}
