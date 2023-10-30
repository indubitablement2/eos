mod client;
pub mod client_connection;
pub mod instance_connection;

use self::instance_connection::*;
use super::*;
use crate::instance_server::*;
use client::*;
use client_connection::*;
use metascape::*;

const TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(100);

pub struct CentralServer {
    next_metascape_id: MetascapeId,
    metascapes: AHashMap<MetascapeId, Metascape>,

    next_client_id: ClientId,
    clients: IndexMap<ClientId, Client, RandomState>,

    client_connection_receiver: std::sync::mpsc::Receiver<Connection>,
    client_login_connections: Vec<Connection>,
    client_connections: IndexMap<ClientId, ClientConnection, RandomState>,

    instance_connection_receiver: std::sync::mpsc::Receiver<Connection>,
    instance_connections: Vec<InstanceConnection>,
}
impl CentralServer {
    pub fn start() {
        Self {
            next_metascape_id: MetascapeId(1),
            metascapes: Default::default(),

            next_client_id: ClientId(0),
            clients: Default::default(),

            client_connection_receiver: Connection::bind_blocking(CENTRAL_ADDR_CLIENT),
            client_login_connections: Default::default(),
            client_connections: Default::default(),

            instance_connection_receiver: Connection::bind_blocking(CENTRAL_ADDR_INSTANCE),
            instance_connections: Default::default(),
        }
        .run();
    }

    fn run(mut self) {
        log::info!("Central server started");

        // TODO: Remove this.
        {
            let metascape = Metascape::new();
            self.metascapes.insert(self.next_metascape_id, metascape);
            self.next_metascape_id.0 += 1;
        }

        let mut interval = Box::pin(tokio::time::interval(TICK_DURATION));
        loop {
            tokio().block_on(interval.tick());
            self.step(0.1);
        }
    }

    fn step(&mut self, delta: f32) {
        // Handle new instance connection.
        for new_connection in self.instance_connection_receiver.try_iter() {
            log::debug!("New connection from instance at {}", new_connection.address);
            self.instance_connections
                .push(InstanceConnection::new(new_connection));
        }

        // Handle new connection.
        for new_connection in self.client_connection_receiver.try_iter() {
            log::debug!("New connection from client at {}", new_connection.address);
            self.client_login_connections.push(new_connection);
        }

        // Handle logins.
        let mut i = 0usize;
        while i < self.client_login_connections.len() {
            let connection = &mut self.client_login_connections[i];

            if let Some(login_packet) = connection.recv::<LoginPacket>() {
                log::debug!("Received {:?}", login_packet);

                // TODO: Handle logins.
                let client_id = ClientId(123);

                let connection = self.client_login_connections.swap_remove(i);
                self.client_connections
                    .insert(client_id, ClientConnection::new(connection, client_id));
            } else if connection.disconnected {
                log::debug!("Client at {} disconnected before login", connection.address);
                self.client_login_connections.swap_remove(i);
            } else {
                i += 1;
            }
        }

        // Handle client packets.
        let mut i = 0usize;
        while i < self.client_connections.len() {
            let connection = &mut self.client_connections[i];

            while let Some(packet) = connection.connection.recv() {
                match packet {
                    ClientPacket::MetascapeCommand { metascape_id, cmd } => {
                        if let Some(metascape) = self.metascapes.get_mut(&metascape_id) {
                            metascape.handle_command(cmd);
                        }
                    }
                }
            }

            // Remove disconnected clients.
            if connection.connection.disconnected {
                log::debug!("{:?} disconnected", connection.client_id);
                self.client_connections.swap_remove_index(i);
            } else {
                i += 1;
            }
        }

        for metascape in self.metascapes.values_mut() {
            metascape.step(delta);
        }

        // Send server packets.
        for connection in self.client_connections.values_mut() {
            if let Some(metascape) = self.metascapes.get(&connection.view.0) {
                let remove_fleets = Vec::new();
                let mut partial_fleets_info = Vec::new();
                let full_fleets_info = Vec::new();
                let mut positions = Vec::new();
                for (&fleet_id, fleet) in metascape.fleets.iter() {
                    positions.push((fleet_id, fleet.position));

                    let known_fleet =
                        connection.knows_fleets.entry(fleet_id).or_insert_with(|| {
                            partial_fleets_info.push((fleet_id, 1));
                            KnownFleet { full_info: false }
                        });
                    // TODO: Maybe send full info.
                }

                connection.connection.send(ServerPacket::State {
                    time: metascape.time_total,
                    partial_fleets_info,
                    full_fleets_info,
                    positions,
                    remove_fleets,
                });
            }
        }
    }
}
