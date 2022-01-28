use std::collections::VecDeque;
use crate::connection_manager::ConnectionsManager;
use crate::server_configs::ClientsManagerConfigs;
use ahash::AHashMap;
use common::net::packets::*;
use common::net::connection::*;
use common::idx::*;

pub enum ConnectError {
    /// The pending queue is empty.
    Empty,
    /// An existing connection was overwriten by a newer one.
    AlreadyConnected,
}

pub struct ClientsManager {
    connection_manager: ConnectionsManager,
    pendings: VecDeque<Connection>,
    connected: AHashMap<ClientId, Connection>,
    last_pendings_update: u32,
    min_pending_for_update: usize,
    pendings_update_interval: u32,
}
impl ClientsManager {
    pub fn new(parameters: &ClientsManagerConfigs) -> std::io::Result<Self> {
        Ok(Self {
            connection_manager: ConnectionsManager::new(parameters.local)?,
            pendings: Default::default(),
            connected: Default::default(),
            last_pendings_update: Default::default(),
            min_pending_for_update: parameters.min_pending_for_update,
            pendings_update_interval: parameters.pendings_update_interval,
        })
    }

    /// TODO: Check for disconnect while sending queue size.
    fn update_pendings(&mut self) {
        
    }

    /// Fetch new connection from the `ConnectionsManager` 
    /// and append them at the end of the pendings queue.
    /// 
    /// Does a pending queue update if nessesary (long queue and minimum duration elapsed).
    pub fn handle_pending_connections(&mut self) {
        while let Ok(new_connection) = self.connection_manager.new_connection_receiver.try_recv() {
            self.pendings.push_back(new_connection);
        }

        if self.pendings.len() > self.min_pending_for_update {
            self.last_pendings_update += 1;

            if self.last_pendings_update > self.pendings_update_interval {
                self.update_pendings();
                self.last_pendings_update = 0;
            }
        } else {
            self.last_pendings_update = 0;
        }
    }

    /// Pop a pending connection and return a mutable reference to it.
    pub fn try_connect_one(&mut self) -> Result<&mut Connection, ConnectError> {
        if let Some(new_connection) = self.pendings.pop_front() {
            match self.connected.try_insert(new_connection.client_id(), new_connection) {
                Ok(new_connection) => Ok(new_connection),
                Err(err) => {
                    // New connection take the place of the old connection.
                    let (_, old_connection) = err.entry.replace_entry(err.value);

                    debug!(
                        "{:?} was disconnected as a new connection took this client.",
                        old_connection.client_id()
                    );

                    // Send message to old client explaining why he got disconnected.
                    old_connection.send_packet_reliable(
                        Packet::DisconnectedReason(DisconnectedReasonEnum::ConnectionFromOther).serialize(),
                    );
                    old_connection.flush_tcp_stream();

                    Err(ConnectError::AlreadyConnected)
                }
            }
        } else {
            Err(ConnectError::Empty)
        }
    }

    /// Remove a connection and return it.
    pub fn remove_connection(&mut self, client_id: ClientId) -> Option<Connection> {
        self.connected.remove(&client_id)
    }

    pub fn get_connection(&self, client_id: ClientId) -> Option<&Connection> {
        self.connected.get(&client_id)
    }
}
