use crate::{connection_manager::{Connection, ConnectionsManager}, data_manager::ClientData, packets::ServerAddresses};
use bevy_ecs::prelude::*;
use indexmap::IndexMap;

pub struct ClientId(pub u32);

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

    pub fn get_addresses(&self) -> ServerAddresses {
        self.connection_manager.get_addresses()
    }
}

pub struct Client {
    pub connection: Connection,

    pub fleet_control: Option<Entity>,

    pub client_data: ClientData,

    // /// What this client's next Battlescape input will be.
    // input_battlescape: BattlescapeInput,
    // /// Resend previous battlescape commands if they have not been acknowledged.
    // unacknowledged_commands: IndexMap<u32, Vec<u8>>,
}
