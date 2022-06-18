use super::*;
use common::fleet::*;
use std::ops::{Deref, DerefMut};

pub struct FleetCompositionMut<'a> {
    inner: &'a mut FleetInner,
    changed: bool,
}
impl Deref for FleetCompositionMut<'_> {
    type Target = FleetComposition;

    fn deref(&self) -> &Self::Target {
        &self.inner.fleet_composition
    }
}
impl DerefMut for FleetCompositionMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        &mut self.inner.fleet_composition
    }
}
impl Drop for FleetCompositionMut<'_> {
    fn drop(&mut self) {
        if self.changed {
            self.inner.last_change = tick();
            self.inner.fleet_stats = self.inner.fleet_composition.compute_stats();
        }
    }
}

/// Has `FleetComposition` and `FleetStats`.
/// Needs to be its own struct to keep track of changes.
pub struct FleetInner {
    fleet_stats: FleetStats,
    fleet_composition: FleetComposition,
    /// The tick this fleet last changed (name, faction_id, composition).
    /// Used for networking & recomputing fleet stats.
    last_change: u32,
}
impl FleetInner {
    pub fn new(fleet_composition: FleetComposition) -> Self {
        Self {
            fleet_stats: fleet_composition.compute_stats(),
            fleet_composition,
            last_change: tick(),
        }
    }

    pub fn fleet_composition(&self) -> &FleetComposition {
        &self.fleet_composition
    }

    /// Will trigger re-computing the fleet's stats if accessed mutably when dropped.
    pub fn fleet_composition_mut(&mut self) -> FleetCompositionMut {
        FleetCompositionMut {
            inner: self,
            changed: false,
        }
    }

    pub fn fleet_stats(&self) -> &FleetStats {
        &self.fleet_stats
    }

    pub fn last_change(&self) -> u32 {
        self.last_change
    }

    pub fn update_stats(&mut self) {
        self.fleet_stats = self.fleet_composition.compute_stats();
    }
}
