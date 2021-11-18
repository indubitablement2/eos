use ahash::AHashMap;

pub struct FactionId(u32);

pub struct Reputation {
    /// General reputation modifier with all factions.
    reputation: i16,
    /// Direct reputation with other faction.
    relation: AHashMap<FactionId, i16>,
}
impl Default for Reputation {
    fn default() -> Self {
        Self { reputation: 0, relation: AHashMap::new() }
    }
}

pub struct FactionsRes {
    factions: AHashMap<FactionId, Faction>,
}
impl FactionsRes {
    pub fn new() -> Self {
        Self {
            factions: AHashMap::new(),
        }
    }
}



struct Faction {
    display_name: String,
    /// Relation with other faction. If a faction is not there, it default to 0 (neutral).
    relation: AHashMap<FactionId, i16>,
}
impl Faction {
    const RELATION_CLAMP: i16 = 10000;
}
