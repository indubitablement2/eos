use super::*;
use std::num::NonZeroU64;

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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct BattlescapeId(pub u64);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct ClientId(pub u64);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct InstanceId(pub u64);
