mod request;
mod save;

use common::connection::*;
use common::database_packet::*;
use common::ids::*;
use common::server::ServerId;
use common::ship::{ShipDataId, ShipId};
use common::system::SystemId;
use common::*;
use sha2::Digest;
use std::time::{Duration, Instant};

const SAVE_INTERVAL: Duration = Duration::from_secs(1 * 60 * 60);

struct Database {
    password: String,

    next_save: Instant,

    restart_request: Option<Instant>,

    connection_listener: ConnectionListener,
    auth_connections: Vec<(Connection, u64)>,

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
    ships: HashSet<ShipId>,
    // TODO: Debris
    // TODO: items
    // TODO: planets state
}

struct Ship {
    ship_data_id: ShipDataId,

    system_id: SystemId,
    position: Vec2,
}

#[derive(Default)]
struct Client {
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

                    let simulations = request
                        .server_id
                        .systems()
                        .into_iter()
                        .map(|system_id| (*system_id, ()))
                        .collect();
                    connection.queue(ServerAuthResponse { simulations });

                    // TODO: Send ships
                }
                false
            } else {
                true
            }
        });

        // Handle requests.
        let mut i = 0;
        while i < self.servers.len() {
            let Some(connection) = self.servers[i]
                .as_mut()
                .map(|server| server.connection.clone())
            else {
                i += 1;
                continue;
            };

            let server_id = ServerId::try_from(i as u32).unwrap();

            while let Some(request) = connection.try_recv::<ServerRequest>() {
                self.handle_request(server_id, request);
            }

            i += 1;
        }

        // Save sometime.
        if self
            .next_save
            .checked_duration_since(Instant::now())
            .is_some()
        {
            self.save();
            self.next_save = Instant::now() + SAVE_INTERVAL;
        }

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
            .is_some_and(|instant| instant.checked_duration_since(Instant::now()).is_some())
    }
}
