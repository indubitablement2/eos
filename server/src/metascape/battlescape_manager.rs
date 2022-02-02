use ahash::AHashMap;
use bevy_ecs::entity::Entity;
use common::idx::*;

#[derive(Debug)]
pub struct Battlescape {
    pub teams: Vec<Vec<Entity>>,
    pub time: u32,
}

#[derive(Debug, Default)]
pub struct BattlescapeManager {
    last_used: u64,
    pub active_battlescape: AHashMap<BattlescapeId, Battlescape>,
}
impl BattlescapeManager {
    pub fn start_new_battlescape(&mut self, teams: Vec<Vec<Entity>>) -> BattlescapeId {
        debug_assert!(
            teams.len() > 1,
            "Created a battlescape with less than 2 teams."
        );

        self.last_used += 1;
        let battlescape_id = BattlescapeId::from_raw(self.last_used);

        let result = self
            .active_battlescape
            .insert(battlescape_id, Battlescape { teams, time: 0 });
        debug_assert!(result.is_none());

        battlescape_id
    }
}
