use ahash::{AHashMap, AHashSet};
use serde::{Deserialize, Serialize};

use crate::{idx::*, reputation::Reputation, systems::Systems};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub name: String,
    #[serde(skip)]
    pub colonies: AHashSet<PlanetId>,
    /// Reputation between factions.
    /// The highest `FactionId` has the reputation of all lower `FactionId`.
    ///
    /// eg: fleet a has faction `2` and fleet b has faction `4`.
    ///
    /// Relation = `faction[4].reputation[2]`.
    pub faction_relation: AHashMap<FactionId, Reputation>,
    /// Fallback reputation.
    pub default_reputation: Reputation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Factions {
    pub factions: AHashMap<FactionId, Faction>,
}
impl Factions {
    pub fn update_all(&mut self, systems: &mut Systems) {
        // Add colonies.
        for (system_id, system) in systems.systems.iter_mut() {
            for (planet, planets_offset) in system.planets.iter_mut().zip(0u8..) {
                if let Some(faction_id) = planet.faction {
                    if let Some(faction) = self.factions.get_mut(&faction_id) {
                        faction.colonies.insert(PlanetId {
                            system_id: *system_id,
                            planets_offset,
                        });
                    } else {
                        planet.faction = None;
                    }
                }
            }
        }

        // TODO: Add relations with other factions.
        // TODO: Remove relation with non-existant faction.
    }
}
