use crate::ecs_components::{ClientId, FactionId, Reputation};
use ahash::AHashMap;
use indexmap::IndexMap;

pub struct Faction {
    pub owner: Option<ClientId>,
    pub display_name: String,
    /// Reputation with individual clients.
    pub clients_relation: AHashMap<ClientId, Reputation>,
    pub base_reputation: Reputation,
}

pub struct FactionsRes {
    pub factions: IndexMap<FactionId, Faction>,
    /// Reputation between faction.
    /// The lowest FactionId is used.
    pub faction_relation: AHashMap<FactionId, Reputation>,
}
impl FactionsRes {
    pub fn new() -> Self {
        Self {
            factions: IndexMap::new(),
            faction_relation: AHashMap::new(),
        }
    }
}
