use crate::res_clients::ClientId;
use bevy_ecs::prelude::Entity;
use indexmap::IndexMap;

/// Never recycled.
/// First 2^32 - 1 idx are reserved for clients.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FleetId(pub u64);
impl From<ClientId> for FleetId {
    fn from(client_id: ClientId) -> Self {
        Self(client_id.0 as u64)
    }
}

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
