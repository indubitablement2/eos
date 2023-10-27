mod client;
pub mod client_connection;

use super::*;
use crate::instance_server::*;
use client::*;
use client_connection::*;
use metascape::*;

pub struct CentralServer {
    next_metascape_id: MetascapeId,
    metascapes: AHashMap<MetascapeId, Metascape>,

    next_client_id: ClientId,
    clients: IndexMap<ClientId, Client, RandomState>,

    client_connection_receiver: std::sync::mpsc::Receiver<Connection<ClientPacket, ServerPacket>>,
    client_login_connections: Vec<Connection<ClientPacket, ServerPacket>>,
    client_connections: IndexMap<ClientId, ClientConnection, RandomState>,

    instance_connection_receiver:
        std::sync::mpsc::Receiver<Connection<InstanceServerPacket, CentralServerPacket>>,
    // instance_connections: IndexMap<ClientId, ClientConnection, RandomState>,
}
impl CentralServer {
    pub fn start() {
        Self {
            next_metascape_id: MetascapeId(1),
            metascapes: Default::default(),

            next_client_id: ClientId(0),
            clients: Default::default(),

            client_connection_receiver: tokio().block_on(Connection::bind_central_client()),
            client_login_connections: Default::default(),
            client_connections: Default::default(),

            instance_connection_receiver: tokio().block_on(Connection::bind_central_instance()),
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

        let mut now = std::time::Instant::now();
        loop {
            self.step(now.elapsed().as_secs_f32());
            now = std::time::Instant::now();
            // TODO: Use a better sleep method.
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn step(&mut self, delta: f32) {
        // Handle new connection.
        for new_connection in self.client_connection_receiver.try_iter() {
            log::debug!("New connection from client at {}", new_connection.address);
            self.client_login_connections.push(new_connection);
        }

        // TODO: Handle logins.

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
                    ClientPacket::Login(_) => {
                        connection.connection.disconnected = true;
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
