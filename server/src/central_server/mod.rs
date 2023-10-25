mod client;
pub mod connection;

use super::*;
use client::*;
use connection::*;
use metascape::*;

type Clients = IndexMap<ClientId, Client, RandomState>;
type Connections = IndexMap<ClientId, Connection, RandomState>;

pub struct CentralServer {
    next_metascape_id: MetascapeId,
    metascapes: AHashMap<MetascapeId, Metascape>,

    next_client_id: ClientId,
    clients: Clients,

    connection_receiver: std::sync::mpsc::Receiver<Connection>,
    connections: Connections,
}
impl CentralServer {
    pub async fn start() {
        let connection_receiver = connection::start_server_loop().await;

        Self {
            next_metascape_id: MetascapeId(1),
            metascapes: Default::default(),

            next_client_id: ClientId(0),
            clients: Default::default(),

            connection_receiver,
            connections: Default::default(),
        }
        .run();
    }

    fn run(mut self) {
        // TODO: Remove this.
        let metascape = Metascape::new();
        self.metascapes.insert(self.next_metascape_id, metascape);
        self.next_metascape_id.0 += 1;

        std::thread::spawn(move || {
            let mut now = std::time::Instant::now();
            loop {
                self.step(now.elapsed().as_secs_f32());
                now = std::time::Instant::now();
                // TODO: Use a better sleep method.
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }

    fn step(&mut self, delta: f32) {
        // Handle new connection.
        for new_connection in self.connection_receiver.try_iter() {
            log::debug!("{:?} connected", new_connection.client_id);
            self.connections
                .insert(new_connection.client_id, new_connection);
        }

        // Handle client packets.
        let mut i = 0usize;
        while i < self.connections.len() {
            let connection = &mut self.connections[i];

            while let Some(packet) = connection.recv() {
                match packet {
                    ClientPacket::MetascapeCommand { metascape_id, cmd } => {
                        if let Some(metascape) = self.metascapes.get_mut(&metascape_id) {
                            metascape.handle_command(cmd);
                        }
                    }
                }
            }

            // Remove disconnected clients.
            if connection.disconnected {
                log::debug!("{:?} disconnected", connection.client_id);
                self.connections.swap_remove_index(i);
            } else {
                i += 1;
            }
        }

        for metascape in self.metascapes.values_mut() {
            metascape.step(delta);
        }

        // Send server packets.
        for connection in self.connections.values_mut() {
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

                connection.send(ServerPacket::State {
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
