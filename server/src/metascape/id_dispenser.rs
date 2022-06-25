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
pub struct FactionIdDispenser(AtomicU64);
impl FactionIdDispenser {
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }
}
impl IdDispenser<FactionId> for FactionIdDispenser {
    fn next(&self) -> FactionId {
        FactionId(self.0.fetch_add(1, Ordering::Relaxed))
    }

    unsafe fn set(&self, id: FactionId) {
        self.0.store(id.0, Ordering::Relaxed);
    }

    unsafe fn current(&self) -> FactionId {
        FactionId(self.0.load(Ordering::Relaxed))
    }
}

#[derive(Debug)]
pub struct FleetIdDispenser(AtomicU64);
impl FleetIdDispenser {
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }
}
impl IdDispenser<FleetId> for FleetIdDispenser {
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
