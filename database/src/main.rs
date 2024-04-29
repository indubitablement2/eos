use common::connection::*;
use common::ids::*;
use common::{HashMap, HashSet, IndexMap};
use flume::{unbounded, Receiver, Sender, TryRecvError};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
enum DatabaseMutation {}

struct Database {
    restart_request: Option<Instant>,

    // mut_requests_writer: Option<BufWriter<File>>,
    connection_listener: ConnectionListener,
    connections: Vec<(ConnectionType, Connection)>,

    instances: IndexMap<InstanceId, Instance>,
    simulations: HashMap<SimulationId, Simulation>,

    next_ship_id: ShipId,
    ships: HashMap<ShipId, Ship>,

    next_client_id: ClientId,
    clients: HashMap<ClientId, Client>,
    username: HashMap<String, ClientId>,
}

enum ConnectionType {
    Client(ClientId),
    Instance(InstanceId),
    Simulation(SimulationId),
}

struct Instance {
    connection: Connection,
    // TODO: perf
}

struct Simulation {
    simulation_save: Vec<u8>,
    ships: HashSet<ShipId>,
    connection: Option<Connection>,
}

struct Ship {
    simulation_id: SimulationId,
    hull_save: Vec<u8>,
}

struct Client {
    password_sha256: Option<Vec<u8>>,
    ships: HashSet<ShipId>,
    connection: Option<Connection>,
}

impl Database {
    fn step(&mut self) -> bool {
        self.restart_request
            .is_some_and(|instant| !instant.saturating_duration_since(Instant::now()).is_zero())
    }

    fn apply_mutation(&mut self, mutation: &mut DatabaseMutation) {
        match mutation {
            _ => todo!(),
        }
    }
}

fn main() {
    let mut database = DatabaseSave::load_database();

    let mut interval = common::interval::Interval::new(100, 500);
    loop {
        interval.step();
        if database.step() {
            break;
        }
    }

    database.save();
}

// ####################################################################################
// ################################### SAVE ###########################################
// ####################################################################################

impl Database {
    fn save(&self) {
        let save = DatabaseSave::V0;

        // TODO: Save save to file
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
enum DatabaseSave {
    #[default]
    V0,
    V1 {},
}
impl DatabaseSave {
    fn to_database(self) -> Result<Database, Self> {
        Err(match self {
            DatabaseSave::V0 => Self::V1 {},
            DatabaseSave::V1 {} => {
                return Ok(Database {
                    restart_request: todo!(),
                    connection_listener: todo!(),
                    connections: todo!(),
                    instances: todo!(),
                    simulations: todo!(),
                    next_ship_id: todo!(),
                    ships: todo!(),
                    next_client_id: todo!(),
                    clients: todo!(),
                    username: todo!(),
                })
            }
        })
    }

    fn load_database() -> Database {
        // TODO: Load save from file
        let mut database_save = DatabaseSave::default();
        loop {
            match database_save.to_database() {
                Ok(db) => return db,
                Err(save) => database_save = save,
            }
        }
    }
}
