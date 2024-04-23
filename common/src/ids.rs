use super::*;
use std::num::NonZeroU64;

pub trait Id {
    fn next(&mut self);
    fn to_u64(&self) -> u64;
    // May panic if value is out of bounds.
    fn from_u64(value: u64) -> Self;
}
impl Id for u64 {
    fn next(&mut self) {
        *self += 1;
    }

    fn to_u64(&self) -> u64 {
        *self
    }

    fn from_u64(value: u64) -> Self {
        value
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
// pub struct EntityId(NonZeroU64);
// impl Default for EntityId {
//     fn default() -> Self {
//         Self(NonZeroU64::MIN)
//     }
// }
// impl Id for EntityId {
//     fn next(&mut self) {
//         self.0 = self.0.checked_add(1).unwrap();
//     }

//     fn to_u64(&self) -> u64 {
//         self.0.get()
//     }

//     fn from_u64(value: u64) -> Self {
//         Self(NonZeroU64::new(value).unwrap())
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ShipId(NonZeroU64);
impl Default for ShipId {
    fn default() -> Self {
        Self(NonZeroU64::MIN)
    }
}
impl Id for ShipId {
    fn next(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn from_u64(value: u64) -> Self {
        Self(NonZeroU64::new(value).unwrap())
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
    fn next(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn from_u64(value: u64) -> Self {
        Self(NonZeroU64::new(value).unwrap())
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
    fn next(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn from_u64(value: u64) -> Self {
        Self(NonZeroU64::new(value).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstanceId(NonZeroU64);
impl Default for InstanceId {
    fn default() -> Self {
        Self(NonZeroU64::MIN)
    }
}
impl Id for InstanceId {
    fn next(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn from_u64(value: u64) -> Self {
        Self(NonZeroU64::new(value).unwrap())
    }
}
