pub mod client;
pub mod client_connection;
pub mod instance_connection;

use self::instance_connection::*;
use super::*;
use crate::battlescape::BattlescapeId;
use crate::instance_server::*;
use client::*;
use client_connection::*;
use metascape::*;

const TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(100);

pub struct CentralServer {
    next_metascape_id: MetascapeId,
    metascapes: AHashMap<MetascapeId, Metascape>,

    next_battlescape_id: BattlescapeId,
    // TODO: Simulated/instance battlescape.
    // TODO: Battlescape state.
    battlescapes: AHashMap<BattlescapeId, ()>,

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

            next_client_id: ClientId(1),
            clients: Default::default(),

            client_connection_receiver: Connection::bind_blocking(CENTRAL_ADDR_CLIENT),
            client_login_connections: Default::default(),
            client_connections: Default::default(),

            instance_connection_receiver: Connection::bind_blocking(CENTRAL_ADDR_INSTANCE),
            instance_connections: Default::default(),

            next_battlescape_id: BattlescapeId(1),
            battlescapes: Default::default(),
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

        let mut last_step = std::time::Instant::now();
        loop {
            let elapsed = last_step.elapsed();
            if let Some(remaining) = TICK_DURATION.checked_sub(elapsed) {
                std::thread::sleep(remaining);
            }

            self.step(last_step.elapsed().as_secs_f32().clamp(
                TICK_DURATION.as_secs_f32() * 0.8,
                TICK_DURATION.as_secs_f32() * 1.4,
            ));
            last_step = std::time::Instant::now();
        }
    }

    fn step(&mut self, delta: f32) {
        // Handle new instance connection.
        for new_connection in self.instance_connection_receiver.try_iter() {
            log::debug!("New connection from instance at {}", new_connection.address);
            self.instance_connections
                .push(InstanceConnection::new(new_connection));
        }

        // Handle instance packets.
        for instance in self.instance_connections.iter_mut() {
            while let Some(packet) = instance.connection.recv::<InstanceCentralPacket>() {
                match packet {
                    InstanceCentralPacket::AuthClient(auth) => {
                        let success = self.client_connections.get(&auth.client_id).is_some_and(
                            |connection| {
                                if connection.token == auth.token {
                                    // TODO: Remove client from other battlescape (if any).
                                    true
                                } else {
                                    false
                                }
                            },
                        );
                        instance
                            .connection
                            .send(CentralInstancePacket::AuthClientResponse { auth, success });
                    }
                }
            }
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
                let mut connection = ClientConnection::new(connection, client_id);

                connection.connection.send(ServerPacket::LoginResponse {
                    token: connection.token,
                    success: true,
                });

                self.client_connections.insert(client_id, connection);
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
                    ClientPacket::MoveFleet {
                        metascape_id,
                        fleet_id,
                        wish_position,
                    } => {
                        if let Some(metascape) = self.metascapes.get_mut(&metascape_id) {
                            metascape.client_move_fleet(
                                connection.client_id,
                                fleet_id,
                                wish_position,
                            );
                        } else {
                            log::debug!(
                                "Client {:?} tried to move fleet {:?} on unknown metascape {:?}",
                                connection.client_id,
                                fleet_id,
                                metascape_id
                            );
                        }
                    }
                    ClientPacket::CreatePracticeBattlescape => {
                        // TODO: Take less busy instance / closest to client.
                        let battlescape_id = self.next_battlescape_id;
                        self.next_battlescape_id.0 += 1;

                        let instance = &mut self.instance_connections[0];

                        self.battlescapes
                            .insert(battlescape_id, instance.create_battlescape(battlescape_id));

                        connection.view = ClientView::Battlescape { battlescape_id };

                        connection
                            .connection
                            .send(ServerPacket::PracticeBattlescapeCreated {
                                battlescape_id,
                                instance_addr: instance.connection.address.ip().to_string(),
                            });
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
            if let ClientView::Metascape {
                metascape_id,
                fleet,
                knows_fleets,
            } = &mut connection.view
            {
                if let Some(metascape) = self.metascapes.get(&metascape_id) {
                    let remove_fleets = Vec::new();
                    let mut partial_fleets_info = Vec::new();
                    let full_fleets_info = Vec::new();
                    let mut positions = Vec::new();
                    for (&fleet_id, fleet) in metascape.fleets.iter() {
                        positions.push((fleet_id, fleet.position));

                        let known_fleet = knows_fleets.entry(fleet_id).or_insert_with(|| {
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
}
