use super::*;
use std::num::NonZeroU64;

pub trait Id: Sized {
    fn next(&mut self) -> Self;
    fn to_u64(&self) -> u64;
    // May panic if value is out of bounds.
    fn try_from_u64(value: u64) -> Option<Self>;
}
impl Id for u64 {
    fn next(&mut self) -> Self {
        let ret = *self;
        *self += 1;
        ret
    }

    fn to_u64(&self) -> u64 {
        *self
    }

    fn try_from_u64(value: u64) -> Option<Self> {
        Some(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(NonZeroU64);
impl Default for ClientId {
    fn default() -> Self {
        Self(NonZeroU64::MIN)
    }
}
impl Id for ClientId {
    fn next(&mut self) -> Self {
        let ret = *self;
        self.0 = self.0.checked_add(1).unwrap();
        ret
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn try_from_u64(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FactionId(NonZeroU64);
impl Default for FactionId {
    fn default() -> Self {
        Self(NonZeroU64::MIN)
    }
}
impl Id for FactionId {
    fn next(&mut self) -> Self {
        let ret = *self;
        self.0 = self.0.checked_add(1).unwrap();
        ret
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn try_from_u64(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ServerId(NonZeroU64);
impl Default for ServerId {
    fn default() -> Self {
        Self(NonZeroU64::MIN)
    }
}
impl Id for ServerId {
    fn next(&mut self) -> Self {
        let ret = *self;
        self.0 = self.0.checked_add(1).unwrap();
        ret
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn try_from_u64(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SimulationId(NonZeroU64);
impl Default for SimulationId {
    fn default() -> Self {
        Self(NonZeroU64::MIN)
    }
}
impl Id for SimulationId {
    fn next(&mut self) -> Self {
        let ret = *self;
        self.0 = self.0.checked_add(1).unwrap();
        ret
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn try_from_u64(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
}
