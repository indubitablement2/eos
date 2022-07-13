use super::*;
use common::fleet::*;
use std::ops::{Deref, DerefMut};

/// Will automatically update the fleet stats when dropped.
pub struct FleetCompositionMut<'a> {
    inner: &'a mut FleetInner,
}
impl Deref for FleetCompositionMut<'_> {
    type Target = FleetComposition;

    fn deref(&self) -> &Self::Target {
        &self.inner.fleet_composition
    }
}
impl DerefMut for FleetCompositionMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.fleet_composition
    }
}
impl Drop for FleetCompositionMut<'_> {
    fn drop(&mut self) {
        self.inner.fleet_stats = self.inner.fleet_composition.compute_stats();
    }
}

/// Has `FleetComposition` and `FleetStats`.
/// Needs to be its own struct to keep track of changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInner {
    fleet_stats: FleetStats,
    fleet_composition: FleetComposition,
    /// The tick this was last changed.
    /// Used for networking & recomputing fleet stats.
    #[serde(skip)]
    change_tick: u32,
}
impl FleetInner {
    pub fn new(fleet_composition: FleetComposition) -> Self {
        Self {
            fleet_stats: fleet_composition.compute_stats(),
            fleet_composition,
            change_tick: Default::default(),
        }
    }

    pub fn fleet_composition(&self) -> &FleetComposition {
        &self.fleet_composition
    }

    /// Will automatically re-compute the fleet's stats, if accessed mutably, when dropped.
    pub fn fleet_composition_mut(&mut self, tick: u32) -> FleetCompositionMut {
        self.change_tick = tick;
        FleetCompositionMut { inner: self }
    }

    pub fn fleet_stats(&self) -> &FleetStats {
        &self.fleet_stats
    }

    /// The tick this was last changed.
    pub fn last_change(&self) -> u32 {
        self.change_tick
    }

    pub fn update_stats(&mut self) {
        self.fleet_stats = self.fleet_composition.compute_stats();
    }
}
