use ahash::AHashMap;
use bevy_ecs::prelude::Entity;
use common::idx::*;

pub struct FleetsRes {
    pub spawned_fleets: AHashMap<FleetId, Entity>,
    last_used_id: u64,
}
impl FleetsRes {
    pub fn new() -> Self {
        Self {
            spawned_fleets: AHashMap::new(),
            last_used_id: u32::MAX as u64,
        }
    }

    /// Get a new unique/never recycled fleet id.
    pub fn get_new_fleet_id(&mut self) -> FleetId {
        self.last_used_id += 1;
        FleetId(self.last_used_id)
    }
}
