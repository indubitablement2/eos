use ahash::AHashMap;
use bevy_ecs::prelude::*;
use common::idx::*;

pub struct FleetsManager {
    spawned_fleets: AHashMap<FleetId, Entity>,
    last_used_id: u64,
}
impl FleetsManager {
    /// Get a new unique/never recycled ai fleet id.
    #[must_use]
    pub fn get_new_fleet_id(&mut self) -> FleetId {
        self.last_used_id += 1;
        FleetId(self.last_used_id)
    }

    pub fn add_spawned_fleet(&mut self, fleet_id: FleetId, entity: Entity) {
        if self.spawned_fleets.insert(fleet_id, entity).is_some() {
            log::error!(
                "{:?} was overwritten. World and fleets manager are out of sync.",
                fleet_id
            );
        }
    }

    pub fn remove_spawned_fleet(&mut self, fleet_id: FleetId) -> Option<Entity> {
        self.spawned_fleets.remove(&fleet_id)
    }

    /// Return the entity of an existing fleet.
    pub fn get_spawned_fleet(&self, fleet_id: FleetId) -> Option<Entity> {
        self.spawned_fleets.get(&fleet_id).copied()
    }
}
impl Default for FleetsManager {
    fn default() -> Self {
        Self {
            spawned_fleets: Default::default(),
            last_used_id: u32::MAX as u64,
        }
    }
}
