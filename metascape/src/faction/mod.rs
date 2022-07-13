pub mod faction_reputations;

pub use self::faction_reputations::*;

use super::*;
use serde_big_array::BigArray;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factions {
    #[serde(with = "BigArray")]
    pub factions: [Faction; 64],
    /// The reputation between factions.
    pub reputations: FactionReputations,
}
impl Factions {
    pub fn get_faction(&self, faction_id: FactionId) -> &Faction {
        &self.factions[faction_id.id() as usize]
    }

    pub fn get_faction_mut(&mut self, faction_id: FactionId) -> &mut Faction {
        &mut self.factions[faction_id.id() as usize]
    }
}
impl Default for Factions {
    fn default() -> Self {
        Self {
            factions: [(); 64].map(|_| Faction::default()),
            reputations: Default::default(),
        }
    }
}

// TODO: Faction trait/affinity that affect reputation with other faction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Faction {
    pub name: String,
    /// Bitmask of enemy and allied factions.
    /// Fleet that are in this faction inherit this.
    pub masks: EnemyAlliedMasks,
    /// Clients that are part of this faction.
    pub clients: AHashSet<ClientId>,
    /// Fleets that are part of this faction.
    pub fleets: AHashSet<FleetId>,
    /// Colonies that are part of this faction.
    pub colonies: AHashSet<PlanetId>,
}
