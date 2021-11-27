use crate::{
    connection_manager::{Connection, ConnectionsManager},
    data_manager::ClientData,
    res_fleets::FleetId,
};
use common::packets::ServerAddresses;
use indexmap::IndexMap;

/// 0 is reserved and mean unvalid/server.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientId(pub u32);
impl ClientId {
    /// Return if this is a valid ClientId, id != 0.
    pub fn is_valid(self) -> bool {
        self.0 != 0
    }
}
impl From<FleetId> for ClientId {
    fn from(fleet_id: FleetId) -> Self {
        Self(fleet_id.0 as u32)
    }
}

pub struct ClientsRes {
    pub connection_manager: ConnectionsManager,
    pub connected_clients: IndexMap<ClientId, Client>,
}
impl ClientsRes {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            connection_manager: ConnectionsManager::new()?,
            connected_clients: IndexMap::new(),
        })
    }

    pub fn get_addresses(&self) -> ServerAddresses {
        self.connection_manager.get_addresses()
    }
}

/// A Client is always controlling the fleet with the same id as the client id.
pub struct Client {
    pub connection: Connection,

    pub client_data: ClientData,
    // /// What this client's next Battlescape input will be.
    // input_battlescape: BattlescapeInput,
    // /// Resend previous battlescape commands if they have not been acknowledged.
    // unacknowledged_commands: IndexMap<u32, Vec<u8>>,
}
