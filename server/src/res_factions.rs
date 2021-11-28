use crate::ecs_components::{ClientId, FactionId};
use ahash::AHashMap;
use indexmap::IndexMap;
use std::ops::{Add, Sub};

/// Good boy points.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reputation(pub i16);
impl Reputation {
    const RELATION_CLAMP: i16 = 1000;
    const ALLIED_THRESHOLD: i16 = 300;
    const ENEMY_THRESHOLD: i16 = -300;

    pub fn is_ally(self) -> bool {
        self.0 > Reputation::ALLIED_THRESHOLD
    }

    pub fn is_enemy(self) -> bool {
        self.0 < Reputation::ENEMY_THRESHOLD
    }
}
impl Default for Reputation {
    fn default() -> Self {
        Self(0)
    }
}
impl Add for Reputation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self((self.0 + rhs.0).min(Reputation::RELATION_CLAMP))
    }
}
impl Sub for Reputation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self((self.0 - rhs.0).max(-Reputation::RELATION_CLAMP))
    }
}

pub struct Faction {
    pub owner: Option<ClientId>,
    pub display_name: String,
    /// Reputation with individual clients.
    pub clients_relation: AHashMap<ClientId, Reputation>,
    pub base_reputation_factionless: Reputation,
    pub base_reputation_faction: Reputation,
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
