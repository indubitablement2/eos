use ahash::{AHashMap, AHashSet};
use bevy_ecs::entity::Entity;
use common::factions::Factions;
use common::idx::*;

#[derive(Debug, Clone, Default)]
pub struct Colony {
    fleets: Vec<Entity>,
    faction: Option<FactionId>,
    population: u64,
}
impl Colony {
    /// Get a reference to the colony's faction.
    pub fn faction(&self) -> Option<FactionId> {
        self.faction
    }
}

#[derive(Debug, Clone, Default)]
pub struct Colonies {
    colonies: AHashMap<PlanetId, Colony>,
    faction_colonies: [AHashSet<PlanetId>; Factions::MAX_FACTIONS],
}
impl Colonies {
    /// Change a colony's faction or create a new colony and give it to the faction.
    pub fn give_colony_to_faction(&mut self, planet_id: PlanetId, new_faction: Option<FactionId>) {
        if let Some(colony) = self.colonies.get_mut(&planet_id) {
            debug_assert_ne!(
                colony.faction, new_faction,
                "Colony is already taken by the same faction."
            );

            // Remove colony's previous faction.
            if let Some(old_faction) = colony.faction {
                let result = self.faction_colonies[old_faction].remove(&planet_id);
                debug_assert!(
                    result,
                    "If a colony has a faction, it's PlanetId should be in faction_colonies"
                );
            }

            // Set colony's new faction.
            colony.faction = new_faction;

            // Add PlanetId to new faction.
            if let Some(faction_id) = new_faction {
                let result = self.faction_colonies[faction_id].insert(planet_id);
                debug_assert!(
                    result,
                    "Faction should not have this PlanetId already in faction_colonies."
                );
            }
        } else {
            // Create a new colony.
            let colony = Colony {
                faction: new_faction,
                ..Default::default()
            };

            // Add PlanetId to new faction.
            if let Some(faction_id) = new_faction {
                let result = self.faction_colonies[faction_id].insert(planet_id);
                debug_assert!(
                    result,
                    "Faction should not have this PlanetId already in faction_colonies."
                );
            }
        }
    }

    /// Get a reference to the colony.
    pub fn colony(&self, planet_id: PlanetId) -> Option<&Colony> {
        self.colonies.get(&planet_id)
    }

    /// Get a reference to the colony's faction if it exist.
    pub fn get_colony_faction(&self, planet_id: PlanetId) -> Option<FactionId> {
        if let Some(colony) = self.colony(planet_id) {
            colony.faction()
        } else {
            None
        }
    }

    /// Get a reference to the faction's colonies.
    pub fn get_faction_colonies(&self, faction_id: FactionId) -> &AHashSet<PlanetId> {
        &self.faction_colonies[faction_id]
    }
}
