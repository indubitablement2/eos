mod server_request;

use common::connection::*;
use common::database_packet::*;
use common::ids::*;
use common::ship::{ShipDataId, ShipId};
use common::*;
use rand::random;
use serde::Serialize;
use sha2::Digest;
use std::time::{Duration, Instant};

const NUM_SERVER_BEFORE_START: usize = 1;
const MAX_DURATION_BEFORE_START: Duration = Duration::from_secs(100);

// TODO: Make opaque
struct Database {
    restart_request: Option<Instant>,

    connection_listener: ConnectionListener,
    pending_connections: Vec<(Connection, u64)>,

    next_sever_id: ServerId,
    servers: HashMap<ServerId, Server>,
    server_connections: Vec<(ServerId, Connection, u64)>,
    server_zones: HashMap<u64, Vec<ServerId>>,

    simulations: HashMap<SimulationId, Simulation>,

    next_ship_id: ShipId,
    ships: HashMap<ShipId, Ship>,

    next_client_id: ClientId,
    clients: HashMap<ClientId, Client>,
    client_connections: Vec<(ClientId, Connection, u64)>,
    username: HashMap<String, ClientId>,
}

struct Server {
    connection: Connection,
    connection_rand_generation: u64,

    zone: u64,
    server_address: String,

    simulations: HashSet<SimulationId>,
    simulation_capacity: f32,
    current_simulation_cost: f32,
    // performance: (),
}

struct Simulation {
    handling_server: ServerId,

    ships: HashSet<ShipId>,
    // TODO: Debris
    // TODO: items
    // TODO: planets state
    connected_clients: HashSet<ClientId>,
}

struct Ship {
    ship_data_id: ShipDataId,

    simulation_id: SimulationId,
    position: Vec2,
}

#[derive(Default)]
struct Client {
    password_sha256: Vec<u8>,
    ships: HashSet<ShipId>,

    connection: Option<Connection>,
    simulation: Option<SimulationId>,
    connection_generation: u64,
}

// ####################################################################################
// ################################### MUTATION #######################################
// ####################################################################################

impl Database {
    fn load() -> Self {
        todo!()
    }

    fn load_simulations(&mut self) {
        todo!()
    }
}

impl Database {
    fn insert_server(
        &mut self,
        given_database_password: String,
        server_address: String,
        simulation_capacity: f32,
        connection: Connection,
    ) -> Option<ServerId> {
        if given_database_password != database_password() {
            return None;
        }

        let zone = server_address
            .get(..2)?
            .as_bytes()
            .iter()
            .enumerate()
            .fold(0u64, |acc, (idx, byte)| acc | ((*byte as u64) << (idx * 8)));

        let server_id = self.next_sever_id.next();
        log::info!("Server authenticated: {}, {}", server_address, zone);

        connection.queue(server_id);

        let connection_rand_generation = random();

        self.servers.insert(
            server_id,
            Server {
                connection: connection.clone(),
                connection_rand_generation,
                zone,
                server_address,
                simulations: Default::default(),
                simulation_capacity,
                current_simulation_cost: 0.0,
            },
        );
        self.server_connections
            .push((server_id, connection, connection_rand_generation));
        self.server_zones.entry(zone).or_default().push(server_id);

        Some(server_id)
    }

    fn remove_server(&mut self, server_id: ServerId) -> Option<Server> {
        let server = self.servers.remove(&server_id)?;

        if let Some(server_zones) = self.server_zones.get_mut(&server.zone) {
            if let Some(idx) = server_zones.iter().position(|&id| id == server_id) {
                server_zones.swap_remove(idx);
            }
        }

        for simulation_id in &server.simulations {
            if let Some(mut simulation) = self.simulations.remove(simulation_id) {
                simulation.handling_server = Default::default();

                for client_id in simulation.connected_clients.drain() {
                    if let Some(client) = self.clients.get_mut(&client_id) {
                        client.simulation = None;
                    }
                }

                self.connect_simulation(*simulation_id, simulation);
            }
        }

        log::warn!("{:?} connection closed", server_id);
        Some(server)
    }

    fn connect_simulation(&mut self, simulation_id: SimulationId, simulation: Simulation) {
        // let mut simulation = self.simulations.remove(&simulation_id)?;
        // if let Some(server) = self.servers.get_mut(&simulation.handling_server) {
        //     server.simulations.remove(&simulation_id);
        // }
        // self.free_simulation.insert(*simulation_id);

        // for client_id in simulation.connected_clients.drain() {
        //     if let Some(client) = self.clients.get_mut(&client_id) {
        //         client.simulation = None;
        //     }
        // }
    }

    fn register_client(&mut self, username: String, password: String) -> Option<ClientId> {
        if password.len() < 8 || password.len() > 32 || username.len() < 4 || username.len() > 32 {
            return None;
        }

        let client_id = match self.username.entry(username) {
            std::collections::hash_map::Entry::Occupied(_) => return None,
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(self.next_client_id.next()).clone()
            }
        };

        let mut hasher = sha2::Sha256::new();
        hasher.update(password.as_bytes());
        let hash = hasher.finalize().to_vec();

        self.clients.insert(
            client_id,
            Client {
                password_sha256: hash,
                ships: Default::default(),
                connection: None,
                simulation: None,
                connection_generation: 0,
            },
        );

        log::debug!("{:?} registered", client_id);
        Some(client_id)
    }

    fn login_client(
        &mut self,
        client_id: ClientId,
        given_password: String,
        connection: Connection,
    ) -> Option<()> {
        let client = self.clients.get(&client_id)?;

        let mut hasher = sha2::Sha256::new();
        hasher.update(given_password.as_bytes());
        let hash = hasher.finalize();
        if &client.password_sha256 != hash.as_slice() {
            return None;
        }

        self.disconnect_client(client_id);

        connection.queue(ClientAuthResponse { client_id });

        let client = self.clients.get_mut(&client_id)?;
        client.connection = Some(connection.clone());
        self.client_connections
            .push((client_id, connection, client.connection_generation));

        log::debug!("{:?} connected", client_id);
        Some(())
    }

    fn disconnect_client(&mut self, client_id: ClientId) -> Option<Connection> {
        let client = self.clients.get_mut(&client_id)?;
        let connection = client.connection.take()?;

        client.connection_generation += 1;

        if let Some(simulation_id) = client.simulation.take() {
            self.disconnect_client_from_simulation(client_id, simulation_id);
        }

        log::debug!("{:?} disconnected", client_id);
        Some(connection)
    }

    fn disconnect_client_from_simulation(
        &mut self,
        client_id: ClientId,
        simulation_id: SimulationId,
    ) -> Option<()> {
        let client = self.clients.get_mut(&client_id)?;
        if client.simulation != Some(simulation_id) {
            return None;
        }
        client.simulation = None;

        let simulation = self.simulations.get_mut(&simulation_id)?;
        simulation.connected_clients.remove(&client_id);

        let server = self.servers.get_mut(&simulation.handling_server)?;
        server.connection.queue(ServerResponse::SimulationResponse {
            simulation_id,
            response: SimulationResponse::ClientAuthorisationRemove { client_id },
        });

        log::debug!("{:?} disconnected from {:?}", client_id, simulation_id);
        Some(())
    }

    fn create_ship(
        &mut self,
        simulation_id: SimulationId,
        ship_data_id: ShipDataId,
        position: Vec2,
    ) -> Option<ShipId> {
        let simulation = self.simulations.get_mut(&simulation_id)?;

        let ship_id = self.next_ship_id.next();

        self.ships.insert(
            ship_id,
            Ship {
                ship_data_id,
                simulation_id,
                position,
            },
        );

        simulation.ships.insert(ship_id);
        if let Some(server) = self.servers.get(&simulation.handling_server) {
            server.connection.queue(ServerResponse::SimulationResponse {
                simulation_id,
                response: SimulationResponse::ShipEnter {
                    ship_id,
                    ship_data_id,
                    position,
                },
            });
        }

        Some(ship_id)
    }

    fn save_ship(
        &mut self,
        simulation_id: SimulationId,
        ship_id: ShipId,
        position: Vec2,
    ) -> Option<()> {
        let ship = self.ships.get_mut(&ship_id)?;
        if ship.simulation_id != simulation_id {
            // TODO: Notify simulation that it doesn't have this ship.
            return None;
        }
        ship.position = position;
        Some(())
    }

    fn move_ship() {
        // todo
    }
}

// ####################################################################################
// ################################### MAIN LOOP ######################################
// ####################################################################################

fn main() {
    common::logger::Logger::init();
    common::load_data();

    let mut database = Database::load();

    log::info!("Database starting");
    let mut interval = common::interval::Interval::new(100, 500);
    let start = Instant::now();
    loop {
        interval.step();
        database.auth();
        if database.servers.len() >= NUM_SERVER_BEFORE_START
            || start.elapsed() > MAX_DURATION_BEFORE_START
        {
            break;
        }
    }

    database.load_simulations();

    log::info!("Database started");
    loop {
        interval.step();
        database.auth();
        if database.step() {
            break;
        }
    }

    log::info!("Database stopping");
}

impl Database {
    fn auth(&mut self) {
        // Take new connections.
        while let Some(connection) = self.connection_listener.try_recv() {
            self.pending_connections.push((connection, 0));
        }

        // Handle authentification.
        let mut pending_connections = std::mem::take(&mut self.pending_connections);
        pending_connections.retain_mut(|(connection, counter)| {
            *counter += 1;
            if let Some(auth) = connection.try_recv::<AuthRequest>() {
                match auth {
                    AuthRequest::Server {
                        database_password,
                        server_address,
                        simulation_capacity,
                    } => {
                        self.insert_server(
                            database_password,
                            server_address,
                            simulation_capacity,
                            connection.clone(),
                        );
                    }
                    AuthRequest::Client {
                        username,
                        password,
                        register,
                    } => {
                        if let Some(&client_id) = self.username.get(&username) {
                            self.login_client(client_id, password, connection.clone());
                        } else if register {
                            if let Some(client_id) =
                                self.register_client(username, password.clone())
                            {
                                self.login_client(client_id, password, connection.clone());
                            }
                        }
                    }
                }
                false
            } else {
                *counter < 100
            }
        });
        self.pending_connections = pending_connections;
    }

    fn step(&mut self) -> bool {
        // todo: Handle client requests.
        let mut client_connections = std::mem::take(&mut self.client_connections);
        client_connections.retain_mut(|(client_id, connection, connection_generation)| {
            if !self
                .clients
                .get(client_id)
                .is_some_and(|client| client.connection_generation == *connection_generation)
            {
                // Old connection
                return false;
            }

            // while let Some(request) = connection.try_recv::<ServerRequest>() {
            //     self.handle_request(*client_id, request);
            // }

            connection.flush();
            if connection.is_closed() {
                self.disconnect_client(*client_id);
                false
            } else {
                true
            }
        });
        self.client_connections = client_connections;

        // Handle server requests.
        let mut server_connections = std::mem::take(&mut self.server_connections);
        server_connections.retain_mut(|(server_id, connection, connection_rand_generation)| {
            if !self.servers.get(server_id).is_some_and(|server| {
                server.connection_rand_generation == *connection_rand_generation
            }) {
                // Old connection
                return false;
            }

            while let Some(request) = connection.try_recv::<ServerRequest>() {
                self.handle_server_request(*server_id, request);
            }

            connection.flush();
            if connection.is_closed() {
                self.remove_server(*server_id);
                false
            } else {
                true
            }
        });
        self.server_connections = server_connections;

        // TODO: wait for all simulation to close and save
        self.restart_request
            .is_some_and(|instant| instant.checked_duration_since(Instant::now()).is_some())
    }
}

// ####################################################################################
// ################################### OTHER ##########################################
// ####################################################################################

#[derive(Serialize)]
struct ClientAuthResponse {
    client_id: ClientId,
}
