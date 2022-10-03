use super::*;

// TODO: Faction trait/affinity that affect reputation with other faction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Faction {
    pub name: String,
    /// Clients that are part of this faction.
    pub clients: AHashSet<ClientId>, // TODO: Use packed map
    /// Fleets that are part of this faction.
    pub fleets: AHashSet<FleetId>, // TODO: Use packed map
    /// Colonies that are part of this faction.
    pub colonies: AHashSet<PlanetId>, // TODO: Use packed map
}


/// Reputation between factions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactionReputations {
    /// The effective reputation between 2 factions.
    /// Use (low, high) as key.
    pub reputations: AHashMap<(FactionId, FactionId), Reputation>,
}
impl FactionReputations {
    pub fn get_reputation_between(&self, a: FactionId, b: FactionId) -> Reputation {
        if let Some(reputation) = self.reputations.get(&(a.min(b), a.max(b))) {
            *reputation
        } else {
            // These factions may not have interacted before.
            Reputation::NEUTRAL
        }
    }

    pub fn get_reputation_between_mut(&mut self, a: FactionId, b: FactionId) -> &mut Reputation {
        self.reputations.entry((a.min(b), a.max(b))).or_default()
    }
}