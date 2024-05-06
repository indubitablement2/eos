mod mutation;
mod save;

use common::connection::*;
use common::database_packet::*;
use common::ids::*;
use common::server::ServerId;
use common::system::SystemId;
use common::{HashMap, HashSet, IndexMap};
use mutation::Mutation;
use rayon::prelude::*;
use sha2::Digest;
use std::time::Instant;
use thread_local::ThreadLocal;

struct Database {
    password: String,

    next_save: Instant,
    save_in_progress: Option<std::thread::JoinHandle<()>>,

    restart_request: Option<Instant>,

    connection_listener: ConnectionListener,
    auth_connections: Vec<(Connection, u64)>,

    mutations: ThreadLocal<std::cell::RefCell<Vec<Mutation>>>,

    /// Indexed by ServerId.
    servers: Vec<Option<Server>>,

    systems: HashMap<SystemId, System>,

    next_ship_id: ShipId,
    ships: HashMap<ShipId, Ship>,

    next_client_id: ClientId,
    clients: HashMap<ClientId, Client>,
    username: HashMap<String, ClientId>,
}

struct Server {
    connection: Connection,
    // performance: (),
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
    connected: Option<SystemId>,
}

// ####################################################################################
// ################################### MAIN LOOP ######################################
// ####################################################################################

fn main() {
    common::logger::Logger::init();
    common::load_data();

    let mut database = Database::load();

    log::info!("Database started");

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
        while let Some(connection) = self.connection_listener.try_recv() {
            self.auth_connections.push((connection, 0));
        }

        // Handle server authentication.
        self.auth_connections.retain_mut(|(connection, counter)| {
            *counter += 1;
            if *counter > 100 {
                false
            } else if let Some(request) = connection.try_recv::<ServerAuthRequest>() {
                if request.password == self.password {
                    log::info!("Server authenticated: {}", request.server_id.ws_addr);
                    self.servers[request.server_id] = Some(Server {
                        connection: connection.clone(),
                    });

                    let system_saves = request
                        .server_id
                        .systems()
                        .into_iter()
                        .map(|system_id| {
                            (system_id, self.systems[&system_id].simulation_save.clone())
                        })
                        .collect();

                    connection.queue(ServerAuthResponse { system_saves });
                }
                false
            } else {
                true
            }
        });

        // Handle requests.
        self.servers
            .par_iter()
            .enumerate()
            .for_each(|(idx, server)| {
                if let Some(server) = server {
                    while let Some(request) = server.connection.try_recv::<ServerRequest>() {
                        self.handle_request(ServerId(&ServerId::data()[idx]), request);
                    }
                }
            });

        // Apply mutations.
        let mut mutations = std::mem::take(&mut self.mutations);
        for mutations in mutations.iter_mut() {
            for mutation in mutations.get_mut().drain(..) {
                self.apply_mutation(mutation);
            }
        }
        for tmp_mutations in self.mutations.iter_mut() {
            mutations
                .get_or_default()
                .borrow_mut()
                .extend(tmp_mutations.borrow_mut().drain(..));
        }
        self.mutations = mutations;

        self.handle_save();

        // Flush connections.
        self.servers.iter_mut().for_each(|maybe_server| {
            if let Some(server) = maybe_server {
                server.connection.flush();

                if server.connection.is_closed() {
                    *maybe_server = None;
                    log::warn!("Server connection closed");
                }
            }
        });

        // TODO: wait for all simulation to close and save
        self.restart_request
            .is_some_and(|instant| !instant.saturating_duration_since(Instant::now()).is_zero())
    }
}
