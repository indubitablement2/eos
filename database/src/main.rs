mod auth_request;
mod mutation;
mod save;

use common::connection::*;
use common::database_packet::*;
use common::ids::*;
use common::system::SystemId;
use common::{HashMap, HashSet, IndexMap};
use rayon::prelude::*;
use sha2::Digest;
use std::net::SocketAddr;
use std::time::Instant;
use thread_local::ThreadLocal;

struct Database {
    password: String,

    next_save: Instant,
    save_in_progress: Option<std::thread::JoinHandle<()>>,

    restart_request: Option<Instant>,

    connection_listener: ConnectionListener,
    connections: Vec<(ConnectionType, Connection)>,

    mutations: ThreadLocal<std::cell::RefCell<Vec<mutation::Mutation>>>,

    next_server_id: ServerId,
    servers: IndexMap<ServerId, Server>,

    queued_systems: Vec<(SystemId, System)>,
    systems: HashMap<SystemId, (System, SystemRunner)>,

    next_ship_id: ShipId,
    ships: HashMap<ShipId, Ship>,

    next_client_id: ClientId,
    clients: HashMap<ClientId, Client>,
    username: HashMap<String, ClientId>,
}

enum ConnectionType {
    Remove,
    Auth { num_iter: u32 },
    Client(ClientId),
    Server(ServerId),
    Simulation(SystemId),
}

struct Server {
    connection: Connection,

    saturation: i32,

    stand_by_runner: Vec<SystemRunner>,
    systems: HashSet<SystemId>,
}

struct SystemRunner {
    connection: Connection,
    client_address: SocketAddr,
}

struct System {
    simulation_save: Option<Vec<u8>>,
    ships: HashSet<ShipId>,
}

struct Ship {
    system_id: SystemId,
    hull_save: Vec<u8>,
}

struct Client {
    password_salt: [u8; 8],
    password_sha256: Option<Vec<u8>>,
    ships: HashSet<ShipId>,
    connection: Option<Connection>,
}

// ####################################################################################
// ################################### MAIN LOOP ######################################
// ####################################################################################

fn main() {
    common::logger::Logger::init();
    common::load_data();
    let mut database = Database::load();

    let mut interval = common::interval::Interval::new(100, 500);
    loop {
        interval.step();
        if database.step() {
            break;
        }
    }

    database.save();
}

impl Database {
    fn step(&mut self) -> bool {
        // Take new connections.
        while let Some(new_connection) = self.connection_listener.try_recv() {
            self.connections
                .push((ConnectionType::Auth { num_iter: 0 }, new_connection));
        }

        // Handle incoming packets.
        let mut connections = std::mem::take(&mut self.connections);
        connections.par_iter_mut().enumerate().for_each(
            |(connection_idx, (connection_type, connection))| {
                self.handle_connection(connection_idx, connection_type, connection);
            },
        );
        self.connections = connections;

        // Apply mutations.
        let mut mutations = std::mem::take(&mut self.mutations);
        for mutations in mutations.iter_mut() {
            for mutation in mutations.get_mut().drain(..) {
                self.apply_mutation(mutation);
            }
        }
        self.mutations = mutations;

        self.handle_save();

        // TODO: Distribute systems to servers based on saturation and location.
        while !self.queued_systems.is_empty() && !self.servers.is_empty() {
            let (_, server) = self.servers.first_mut().unwrap();
            if let Some(runner) = server.stand_by_runner.pop() {
                let (system_id, system) = self.queued_systems.pop().unwrap();
                runner.connection.queue(system_id);

                self.connections.push((
                    ConnectionType::Simulation(system_id),
                    runner.connection.clone(),
                ));
                self.systems.insert(system_id, (system, runner));
                server.systems.insert(system_id);
            }
        }

        // Flush connections.
        self.connections
            .retain(|(connection_type, connection)| match connection_type {
                ConnectionType::Remove => false,
                ConnectionType::Auth { num_iter } => *num_iter < 256,
                ConnectionType::Client(_) => {
                    connection.flush();
                    if connection.is_closed() {
                        // TODO:
                        false
                    } else {
                        true
                    }
                }
                ConnectionType::Server(_) => {
                    connection.flush();
                    if connection.is_closed() {
                        // TODO:
                        false
                    } else {
                        true
                    }
                }
                ConnectionType::Simulation(_) => {
                    connection.flush();
                    if connection.is_closed() {
                        // TODO:
                        false
                    } else {
                        true
                    }
                }
            });

        // TODO: wait for all simulation to close and save
        self.restart_request
            .is_some_and(|instant| !instant.saturating_duration_since(Instant::now()).is_zero())
    }

    fn handle_connection(
        &self,
        connection_idx: usize,
        connection_type: &mut ConnectionType,
        connection: &Connection,
    ) {
        match connection_type {
            ConnectionType::Remove => {}
            ConnectionType::Auth { num_iter } => {
                if let Some(request) = connection.try_recv::<AuthRequest>() {
                    *num_iter += 1;
                    *connection_type = ConnectionType::Remove;
                    if let Some(mutation) =
                        self.handle_auth_request(request, connection_idx, connection)
                    {
                        self.push_mutation(mutation);
                    }
                }
            }
            ConnectionType::Client(id) => todo!(),
            ConnectionType::Server(id) => todo!(),
            ConnectionType::Simulation(id) => todo!(),
        }
    }
}
