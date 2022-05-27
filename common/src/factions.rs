use crate::{idx::*, reputation::Reputation};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub name: String,
    /// Reputation with other factions.
    /// This is the unique id of the reputation.
    pub reputations: AHashMap<FactionId, usize>,
    /// Used when 2 faction don't have explicit reputation 
    /// (eg. when they have never interacted before).
    pub default_reputation: Reputation,
    // TODO: Keep track of Players/fleet in this faction.
    // TODO: When empty, clean up the faction.
    // pub players: AHashSet<PlanetId>,
}
impl Default for Faction {
    fn default() -> Self {
        Self {
            name: "Independent".to_string(),
            reputations: Default::default(),
            default_reputation: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factions {
    next_faction_id: FactionId,
    factions: AHashMap<FactionId, Faction>,

    next_reputation_id: usize,
    free_reputation_id: Vec<usize>,
    reputations: Vec<Reputation>,
}
impl Factions {
    pub fn create_faction(&mut self, faction: Faction) -> FactionId {
        let faction_id = self.next_faction_id;
        self.next_faction_id.0 += 1;

        self.factions.insert(faction_id, faction);

        faction_id
    }

    /// Return the reputation between 2 factions.
    /// 
    /// If a faction does not exist, log an error and return a default value.
    pub fn get_reputations_between(&self, a: FactionId, b: FactionId) -> Reputation {
        if a == b {
            return Reputation::MAX;
        }

        if let Some(faction_a) = self.factions.get(&a) {
            let id = if let Some(id) = faction_a.reputations.get(&b) {
                id
            } else {
                return Reputation::default();
            };

            self.reputations.get(*id).copied().unwrap_or_default()
        } else {
            log::warn!("Unkow faction {:?} for `get_reputations_between`. Returning default...", a);
            Reputation::default()
        }
    }

    pub fn get_faction(&self, faction_id: FactionId) -> Option<&Faction> {
        self.factions.get(&faction_id)
    }

    pub fn get_reputation(&self, id: usize) -> Option<&Reputation> {
        self.reputations.get(id)
    }

    /// Will return a default value if it does not exist. 
    pub fn get_reputation_infaillible(&self, id: usize) -> Reputation {
        self.reputations.get(id).copied().unwrap_or_default()
    }
}
