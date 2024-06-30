pub mod client;
mod save;
pub mod server;
pub mod ship;
pub mod simulation;

use common::connection::*;
use common::database_packet::*;
use common::ids::*;
use common::ship::{ShipDataId, ShipId};
use common::*;
use rand::random;
use save::*;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::time::{Duration, Instant};

const MAX_SERVER_BEFORE_START: usize = 1;
const MAX_DURATION_BEFORE_START: Duration = Duration::from_secs(200);

pub const fn database_password() -> &'static str {
    std::env!("DATABASE_PASSWORD")
}

pub struct Database {
    restart_request: Option<Instant>,

    saver: Option<Saver>,

    connection_listener: ConnectionListener,
    pending_connections: Vec<(Connection, u64)>,

    next_sever_id: ServerId,
    servers: HashMap<ServerId, Server>,
    server_connections: Vec<(ServerId, Connection, u64)>,
    server_zones: HashMap<u64, Vec<ServerId>>,

    next_client_id: ClientId,
    clients: HashMap<ClientId, Client>,
    client_connections: Vec<(ClientId, Connection, u64)>,
    username: HashMap<String, ClientId>,

    next_simulation_id: SimulationId,
    simulations: HashMap<SimulationId, Simulation>,

    next_ship_id: ShipId,
    ships: HashMap<ShipId, Ship>,
}

pub struct Server {
    connection: Connection,
    connection_rand_generation: u64,

    zone: u64,
    server_address: String,

    simulations: HashSet<SimulationId>,
    simulation_capacity: u32,
    current_simulation_cost: u32,
    // performance: (),
}

pub struct Client {
    pub auth_level: ClientAuthLevel,
    username: String,
    password_sha256: [u8; 32],
    ships: HashSet<ShipId>,

    connection: Option<Connection>,
    simulation: Option<SimulationId>,
    connection_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum ClientAuthLevel {
    #[default]
    User,
    Admin,
    SuperAdmin,
}

pub struct Simulation {
    handling_server: ServerId,
    connected_clients: HashSet<ClientId>,

    position: Vec2,
    zone: u64,

    ships: HashSet<ShipId>,
    // TODO: Debris
    // TODO: items
    // TODO: planets
}

pub struct Ship {
    ship_data_id: ShipDataId,

    owner: Option<ClientId>,

    simulation_id: SimulationId,
    position: Vec2,
}

impl Database {
    pub fn start() {
        let mut database = Self {
            restart_request: None,
            saver: None,
            connection_listener: ConnectionListener::bind(database_address()).unwrap(),
            pending_connections: Default::default(),
            next_sever_id: Default::default(),
            servers: Default::default(),
            server_connections: Default::default(),
            server_zones: Default::default(),
            next_simulation_id: Default::default(),
            simulations: Default::default(),
            next_ship_id: Default::default(),
            ships: Default::default(),
            next_client_id: Default::default(),
            clients: Default::default(),
            client_connections: Default::default(),
            username: Default::default(),
        };

        log::info!("Database starting");
        let mut interval = common::interval::Interval::new(100, 500);
        let start = Instant::now();
        loop {
            interval.step();
            database.auth(true);
            if database.servers.len() >= MAX_SERVER_BEFORE_START
                || start.elapsed() > MAX_DURATION_BEFORE_START
            {
                break;
            }
        }

        save::Saver::start_and_populate(&mut database);

        log::info!("Database started");
        loop {
            interval.step();
            database.auth(false);
            if database.step() {
                break;
            }
        }

        log::info!("Database stopping");
        database.saver.unwrap().close();
    }

    fn auth(&mut self, server_only: bool) {
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
                        if !server_only {
                            if let Some(&client_id) = self.username.get(&username) {
                                self.connect_client(client_id, &password, connection.clone());
                            } else if register {
                                if let Some(client_id) = self.register_client(username, &password) {
                                    self.connect_client(client_id, &password, connection.clone());
                                }
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

            while let Some(request) = connection.try_recv::<super::client_request::ClientRequest>()
            {
                self.handle_client_request(*client_id, request);
            }

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
