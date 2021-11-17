use bevy_ecs::prelude::Entity;
use indexmap::IndexMap;

pub struct FleetId(u32);

pub struct FleetsRes {
    spawned_fleets: IndexMap<FleetId, Entity>,
}
impl FleetsRes {
    pub fn new() -> Self {
        Self {
            spawned_fleets: IndexMap::new(),
        }
    }

    pub fn spawn_fleet(&mut self) {
        todo!()
    }
}
