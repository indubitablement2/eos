use crate::connection_manager::ConnectionsManager;
use common::connection::Connection;
use common::idx::*;
use indexmap::IndexMap;

pub struct ClientsRes {
    pub connection_manager: ConnectionsManager,
    pub connected_clients: IndexMap<ClientId, Client>,
}
impl ClientsRes {
    pub fn new(local: bool) -> std::io::Result<Self> {
        Ok(Self {
            connection_manager: ConnectionsManager::new(local)?,
            connected_clients: IndexMap::new(),
        })
    }
}

/// A Client is always controlling the fleet with the same id as the client id.
pub struct Client {
    pub connection: Connection,
    // /// What this client's next Battlescape input will be.
    // input_battlescape: BattlescapeInput,
    // /// Resend previous battlescape commands if they have not been acknowledged.
    // unacknowledged_commands: IndexMap<u32, Vec<u8>>,
}
