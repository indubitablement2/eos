use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub trait IdDispenser<T> {
    /// Get the next id and increment the inner counter.
    fn next(&self) -> T;
    /// Set the next id.
    unsafe fn set(&self, id: T);
    /// Get the current id without incrementing the inner counter.
    unsafe fn current(&self) -> T;
}

#[derive(Debug)]
pub struct NPCFleetIdDispenser(AtomicU64);
impl NPCFleetIdDispenser {
    pub const fn new() -> Self {
        Self(AtomicU64::new(u32::MAX as u64 + 1))
    }
}
impl IdDispenser<FleetId> for NPCFleetIdDispenser {
    fn next(&self) -> FleetId {
        FleetId(self.0.fetch_add(1, Ordering::Relaxed))
    }

    unsafe fn set(&self, id: FleetId) {
        self.0.store(id.0, Ordering::Relaxed);
    }

    unsafe fn current(&self) -> FleetId {
        FleetId(self.0.load(Ordering::Relaxed))
    }
}
