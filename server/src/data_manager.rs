use crate::{res_clients::*, res_factions::*};

pub struct DataManager {}
impl DataManager {
    pub fn new() -> Self {
        Self {}
    }

    /// TODO: Load client from file.
    pub fn load_client(&self, client_id: ClientId) -> ClientData {
        ClientData {
            fleet: FleetData::default(),
        }
    }
}

pub struct ClientData {
    fleet: FleetData,
}

pub struct FleetData {
    reputation: Reputation,
}
impl Default for FleetData {
    fn default() -> Self {
        Self {
            reputation: Default::default(),
        }
    }
}
