use ahash::AHashMap;
use common::idx::ClientId;

#[derive(Debug, Default)]
pub struct ClientData {}

pub struct DataManager {
    pub client_fleets: AHashMap<ClientId, ()>,
    /// Friends, preferences and other personal data.
    pub clients_data: AHashMap<ClientId, ClientData>,
}
impl DataManager {
    pub fn new() -> Self {
        Self {
            client_fleets: AHashMap::new(),
            clients_data: AHashMap::new(),
        }
    }
}
