use bevy_ecs::prelude::Entity;
use indexmap::IndexMap;

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FleetId(u64);

pub struct FleetsRes {
    pub spawned_fleets: IndexMap<FleetId, Entity>,
}
impl FleetsRes {
    pub fn new() -> Self {
        Self {
            spawned_fleets: IndexMap::new(),
        }
    }

    pub fn spawn_fleet(&mut self, fleet_id: FleetId,) {
        todo!()
    }
}
