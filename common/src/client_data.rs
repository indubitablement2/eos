use super::*;
use ids::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientData {
    pub client_id: ClientId,
    pub username: String,
    pub leagues: HashMap<LeagueId, ClientLeagueData>,
    pub characters: HashMap<ClientCharacterId, ClientCharacterData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientLeagueData {
    pub league_id: LeagueId,
    // TODO: stash, progress.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCharacterData {
    pub client_character_id: ClientCharacterId,
    pub name: String,
    pub league_id: LeagueId,
    pub health_relative: f32,
    pub experience: u64,
    // TODO: Inventory, equipment, appearance.
}
impl Default for ClientCharacterData {
    fn default() -> Self {
        Self {
            client_character_id: Default::default(),
            name: String::new(),
            health_relative: 1.0,
            experience: 0,
            league_id: Default::default(),
        }
    }
}
