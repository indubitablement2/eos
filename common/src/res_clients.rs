use crate::{
    connection_manager::{Connection, ConnectionsManager},
    packets::ServerAddresses,
    res_factions::FactionId,
};
use ahash::AHashMap;
use bevy_ecs::prelude::Entity;
use indexmap::IndexMap;

pub struct ClientId(u32);

pub struct ClientsRes {
    connection_manager: ConnectionsManager,
    clients: AHashMap<ClientId, Client>,
    connected_clients: IndexMap<ClientId, ConnectedClient>,
}
impl ClientsRes {
    pub fn new(local: bool) -> std::io::Result<Self> {
        Ok(Self {
            connection_manager: ConnectionsManager::new(local)?,
            clients: AHashMap::new(),
            connected_clients: IndexMap::new(),
        })
    }

    pub fn get_addresses(&self) -> ServerAddresses {
        self.connection_manager.get_addresses()
    }
}

struct Client {
    /// Impact every owned fleet reputation.
    reputation: i16,
    /// Impact every owned fleet relation.
    relation: AHashMap<FactionId, i16>,
}

struct ConnectedClient {
    fleet_control: Option<Entity>,

    connection: Connection,
    // /// What this client's next Battlescape input will be.
    // input_battlescape: BattlescapeInput,
    // /// Resend previous battlescape commands if they have not been acknowledged.
    // unacknowledged_commands: IndexMap<u32, Vec<u8>>,
}
