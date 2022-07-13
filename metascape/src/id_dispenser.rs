use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Dispense unique and never recycled `FleetId`.
#[derive(Debug, Serialize, Deserialize)]
pub struct FleetIdDispenser(AtomicU64);
impl FleetIdDispenser {
    /// Get the next npc fleet id and increment the inner counter.
    fn next(&self) -> FleetId {
        FleetId(self.0.fetch_add(1, Ordering::Relaxed))
    }
}
impl Default for FleetIdDispenser {
    fn default() -> Self {
        // First u32::MAX are reserved for client.
        Self(AtomicU64::new(u32::MAX as u64 + 1))
    }
}
