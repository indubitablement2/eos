use crate::ecs_components::FleetId;
use bevy_ecs::prelude::Entity;
use indexmap::IndexMap;

pub struct FleetsRes {
    pub spawned_fleets: IndexMap<FleetId, Entity>,
}
impl FleetsRes {
    pub fn new() -> Self {
        Self {
            spawned_fleets: IndexMap::new(),
        }
    }
}
