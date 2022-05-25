use ahash::{AHashMap, AHashSet};
use common::{idx::*, reputation::Reputation};
use serde::{Deserialize, Serialize};
use utils::*;

// TODO: Faction trait/affinity that affect reputation with other faction.
#[derive(Debug, Clone, Serialize, Deserialize, Fields, Columns, Components)]
pub struct Faction {
    pub name: String,
    /// Reputation with other factions.
    ///
    /// The reputation between 2 factions is stored
    /// in the faction with the lowest FactionId.
    pub reputations: AHashMap<FactionId, Reputation>,
    /// Used when 2 faction don't have explicit reputation
    /// (eg. when they have not interacted before and one is new).
    pub fallback_reputation: Reputation,
    /// Clients that are part of this faction.
    pub clients: AHashSet<ClientId>,
    /// Fleets that are part of this faction.
    pub fleets: AHashSet<FleetId>,
    /// Colonies that are part of this faction.
    pub colonies: AHashSet<PlanetId>,
}
impl Default for Faction {
    fn default() -> Self {
        Self {
            name: "Independent".to_string(),
            reputations: Default::default(),
            fallback_reputation: Default::default(),
            clients: Default::default(),
            fleets: Default::default(),
            colonies: Default::default(),
        }
    }
}

pub struct FactionBuilder {
    pub name: String,
    pub reputations: AHashMap<FactionId, Reputation>,
    pub fallback_reputation: Reputation,
    pub clients: AHashSet<ClientId>,
    pub fleets: AHashSet<FleetId>,
    pub colonies: AHashSet<PlanetId>,
}
impl FactionBuilder {
    pub fn new() -> Self {
        Self {
            name: "Independent".to_string(),
            reputations: Default::default(),
            fallback_reputation: Default::default(),
            clients: Default::default(),
            fleets: Default::default(),
            colonies: Default::default(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Also insert the equivalent FleetIdx.
    pub fn with_clients(mut self, clients: &[ClientId]) -> Self {
        self.clients.extend(clients.iter());
        self.fleets
            .extend(clients.iter().map(|client_id| client_id.to_fleet_id()));
        self
    }

    pub fn build(self) -> Faction {
        Faction {
            name: self.name,
            reputations: self.reputations,
            fallback_reputation: self.fallback_reputation,
            clients: self.clients,
            fleets: self.fleets,
            colonies: self.colonies,
        }
    }
}
