use super::*;
use std::num::{NonZeroU32, NonZeroU64};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(pub NonZeroU64);
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
}
impl Default for EntityId {
    fn default() -> Self {
        Self::from_u64(1).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BattlescapeId(pub NonZeroU64);
impl BattlescapeId {
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
impl Default for BattlescapeId {
    fn default() -> Self {
        Self::from_u64(1).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(pub NonZeroU64);
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
pub struct InstanceId(pub NonZeroU32);
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
