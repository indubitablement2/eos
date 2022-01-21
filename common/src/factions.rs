use crate::{idx::*, reputation::Reputation, systems::Systems};
use ahash::{AHashMap, AHashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub relations: Vec<Reputation>,
    /// Reputation with individual fleet.
    pub reputations: AHashMap<FleetId, Reputation>,
    pub default_reputation: Reputation,

    pub target_colonies: usize,
}
impl Default for Faction {
    fn default() -> Self {
        Self {
            disabled: true,
            name: Default::default(),
            colonies: Default::default(),
            relations: Default::default(),
            reputations: Default::default(),
            default_reputation: Default::default(),
            target_colonies: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Factions {
    pub factions: [Faction; 32],
    /// The enemy mask of each factions.
    #[serde(skip)]
    pub enemy_masks: [u32; 32],
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
            current.relations.resize(i, Reputation::NEUTRAL);
        }
    }

    /// Masks are stored in `enemy_masks`.
    pub fn update_factions_enemy_mask(&mut self) {
        let masks = &mut self.enemy_masks;

        for (i, faction) in self.factions.iter().enumerate() {
            let current = &mut masks[i];

            // Add relation from lower faction.
            for (j, rep) in faction.relations.iter().enumerate() {
                *current += (rep.is_enemy() as u32) << j;
            }

            // Add relation from other higher factions.
            for j in i + 1..self.factions.len() {
                *current += (self.factions[j].relations[i].is_enemy() as u32) << j
            }
        }
    }
}

#[test]
fn test_get_factions_enemy_mask() {
    let mut rng = rand::thread_rng();

    let mut f = Factions::default();
    let mut s = Systems::default();
    f.update_all(&mut s);

    for faction in f.factions.iter_mut() {
        for reputation in faction.relations.iter_mut() {
            *reputation = Reputation(rand::Rng::gen_range(&mut rng, Reputation::MIN.0..Reputation::MAX.0));
        }
    }

    f.update_factions_enemy_mask();

    for (i, mask) in f.enemy_masks.iter().enumerate() {
        println!("{:2} - {:032b}", i, mask);
    }

    for (i, mask) in f.enemy_masks.iter().enumerate() {
        for j in 0..f.factions.len() {
            let is_enemy = (mask & (1 << j)) != 0;

            let (min, max) = if i > j {
                (j, i)
            } else if i < j {
                (i, j)
            } else {
                assert!(!is_enemy, "enemy with itself");
                continue;
            };

            assert_eq!(is_enemy, f.factions[max].relations[min].is_enemy(), "{}, {}", min, max);
        }
    }
}
