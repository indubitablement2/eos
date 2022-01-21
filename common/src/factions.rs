use crate::{idx::*, reputation::Reputation, systems::Systems};
use ahash::AHashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Faction {
    pub disabled: bool,

    pub name: String,
    #[serde(skip)]
    pub colonies: AHashSet<PlanetId>,
    /// Reputation between factions.
    /// The highest `FactionId` has the reputation of all lower `FactionId`.
    ///
    /// eg: fleet a has faction `2` and fleet b has faction `4`.
    ///
    /// Relation = `faction[4].reputation[2]`.
    pub relation: Vec<Reputation>,

    pub target_colonies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Factions {
    pub factions: [Faction; 32],
}
impl Factions {
    pub fn update_all(&mut self, systems: &mut Systems) {
        // Add colonies.
        for (system, id) in systems.systems.iter_mut().zip(0u16..) {
            for (planet, planets_offset) in system.planets.iter_mut().zip(0u8..) {
                if let Some(faction_id) = planet.faction {
                    let faction = &mut self.factions[faction_id];
                    if faction.disabled {
                        planet.faction = None;
                    } else {
                        faction.colonies.insert(PlanetId {
                            system_id: SystemId(id),
                            planets_offset,
                        });
                    }
                }
            }
        }

        for i in 0..self.factions.len() {
            let current = &mut self.factions[i];

            // Add relations with other factions.
            current.relation.resize(i, Reputation::NEUTRAL);
        }
    }
}
