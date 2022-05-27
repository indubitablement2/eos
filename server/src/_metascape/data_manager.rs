use ahash::AHashMap;
use common::idx::ClientId;

#[derive(Debug, Default)]
pub struct ClientData {}

pub struct DataManager {
    /// Friends, preferences and other personal data.
    pub clients_data: AHashMap<ClientId, ClientData>,
}
impl DataManager {
    pub fn new() -> Self {
        Self {
            clients_data: AHashMap::new(),
        }
    }
}
