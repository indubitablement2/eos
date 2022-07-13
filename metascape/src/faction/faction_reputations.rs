use super::*;
use serde_big_array::BigArray;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyAlliedMasks {
    /// Factions that are enemy `1`.
    pub enemy: u64,
    /// Factions that are allied `1`.
    pub allied: u64,
}
impl EnemyAlliedMasks {
    pub fn is_allied_with(&self, other_faction_id: FactionId) -> bool {
        self.allied & other_faction_id.mask() != 0
    }

    pub fn is_enemy_with(&self, other_faction_id: FactionId) -> bool {
        self.allied & other_faction_id.mask() != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionReputations {
    #[serde(with = "BigArray")]
    reputations: [Reputation; 64 * 64],
}
impl FactionReputations {
    /// If a == b, return a default value.
    ///
    /// Otherwise return the reputation between faction a and b.
    pub fn get_reputation_between(&self, a: FactionId, b: FactionId) -> Reputation {
        self.get_reputation_between_internal(a.id() as usize, b.id() as usize)
    }

    pub fn get_reputation_between_internal(&self, a: usize, b: usize) -> Reputation {
        self.reputations[((a << 6) + b)]
    }

    /// If a == b, nothing is done and an error is logged.
    ///
    /// Otherwise set the reputation between faction a and b.
    pub fn set(&mut self, a: FactionId, b: FactionId, value: Reputation) {
        if a == b {
            log::warn!(
                "Tried to change reputation for {:?} between itself. Ignoring...",
                a
            );
            return;
        }
        self.reputations[((a.id() << 6) + b.id()) as usize] = value;
        self.reputations[((b.id() << 6) + a.id()) as usize] = value;
    }

    /// Return the enemy/allied masks of all faction.
    pub fn compute_masks(&self) -> [EnemyAlliedMasks; 64] {
        let mut masks = [EnemyAlliedMasks::default(); 64];

        for a in 0..64 {
            for b in 0..64 {
                match self.get_reputation_between_internal(a, b).state() {
                    ReputationState::Allied => {
                        masks[a].allied |= 1 << b;
                    }
                    ReputationState::Neutral => {}
                    ReputationState::Enemy => {
                        masks[a].enemy |= 1 << b;
                    }
                }
            }
        }

        masks
    }
}

impl Default for FactionReputations {
    fn default() -> Self {
        let mut reputations = [Reputation::NEUTRAL; 64 * 64];
        // Set the reputation with same faction to max.
        for i in 0..64 {
            reputations[(i << 6) + i] = Reputation::MAX;
        }
        Self { reputations }
    }
}
