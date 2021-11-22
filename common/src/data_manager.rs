use crate::{res_clients::*, res_factions::*, res_fleets::*};
use indexmap::IndexMap;

pub struct DataManager {}
impl DataManager {
    pub fn new() -> Self {
        Self {}
    }

    /// TODO: Load client from file.
    pub fn load_client(&self, client_id: ClientId) -> ClientData {
        ClientData {
            general_reputation: Reputation::default(),
            fleets: IndexMap::new(),
        }
    }
}

pub struct ClientData {
    general_reputation: Reputation,
    fleets: IndexMap<FleetId, FleetData>,
}

pub struct FleetData {
    reputation: Reputation,
}
