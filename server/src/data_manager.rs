use common::idx::*;

pub struct DataManager {}
impl DataManager {
    pub fn new() -> Self {
        Self {}
    }

    /// TODO: Load client data from file.
    pub fn load_client(&self, client_id: ClientId) -> Option<ClientData> {
        None
    }

    /// TODO: Load fleet data from file.
    pub fn load_fleet(&self, fleet_id: FleetId) -> Option<FleetData> {
        None
    }
}

pub struct ClientData {}

pub struct FleetData {}
