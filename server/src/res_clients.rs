use crate::connection_manager::ConnectionsManager;
use ahash::AHashMap;
use common::connection::Connection;
use common::idx::*;

pub struct ClientsRes {
    pub connection_manager: ConnectionsManager,
    pub connected_clients: AHashMap<ClientId, Connection>,
}
impl ClientsRes {
    pub fn new(local: bool) -> std::io::Result<Self> {
        Ok(Self {
            connection_manager: ConnectionsManager::new(local)?,
            connected_clients: AHashMap::new(),
        })
    }
}
