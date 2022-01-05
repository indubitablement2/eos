use crate::connection_manager::ConnectionsManager;
use crate::ecs_components::ClientFleetAI;
use ahash::AHashMap;
use common::connection::Connection;
use common::idx::*;

pub struct ClientsRes {
    pub connection_manager: ConnectionsManager,
    pub connected_clients: AHashMap<ClientId, Connection>,
    pub clients_data: AHashMap<ClientId, ClientData>,
}
impl ClientsRes {
    pub fn new(local: bool) -> std::io::Result<Self> {
        Ok(Self {
            connection_manager: ConnectionsManager::new(local)?,
            connected_clients: AHashMap::new(),
            clients_data: AHashMap::new(),
        })
    }
}

#[derive(Debug, Default)]
pub struct ClientData {
    pub client_fleet_ai: ClientFleetAI,
}
