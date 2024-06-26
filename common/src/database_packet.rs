use self::{
    connection::Connection,
    ids::*,
    ship::{ShipDataId, ShipId},
};
use super::*;
use flume::Receiver;
use std::sync::{atomic::AtomicBool, Arc};

// ####################################################################################
// ################################### SERVER AUTH ####################################
// ####################################################################################

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthRequest {
    Server {
        database_password: String,
        server_address: String,
        simulation_capacity: f32,
    },
    Client {
        username: String,
        password: String,
        /// Try to register if the username does not exist.
        /// Otherwise try to login.
        register: bool,
    },
}

// ####################################################################################
// ################################### SERVER #########################################
// ####################################################################################

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerRequest {
    PerfStats {},
    SimulationRequest {
        simulation_id: SimulationId,
        request: SimulationRequest,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerResponse {
    StartSimulation {
        simulation_id: SimulationId,
        // TODO: planets, ships, etc.
    },
    SimulationResponse {
        simulation_id: SimulationId,
        response: SimulationResponse,
    },
    Restart,
}

// ####################################################################################
// ################################### SIMULATION #####################################
// ####################################################################################

#[derive(Debug, Serialize, Deserialize)]
pub enum SimulationRequest {
    ClientLogoff {
        client_id: ClientId,
    },
    Save {
        // simulation_save: Vec<u8>,
        ship_saves: Vec<(ShipId, Vec2)>,
    },
    SaveShip {
        ship_id: ShipId,
        position: Vec2,
    },
    // TODO: ShipChangeSystem
    // TODO: ShipDestroyed
    CreateShip {
        ship_data_id: ShipDataId,
        position: Vec2,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SimulationResponse {
    ClientAuthorisationAdd {
        client_id: ClientId,
        token: u64,
    },
    ClientAuthorisationRemove {
        client_id: ClientId,
    },
    ShipEnter {
        ship_id: ShipId,
        ship_data_id: ShipDataId,
        position: Vec2,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientUpdate {
    pub ships_delta: Vec<()>,
}

pub struct SimulationConnection {
    simulation_id: SimulationId,
    /// Used to send requests only.
    database_connection: Connection,
    database_response_receiver: Receiver<SimulationResponse>,
    new_client_receiver: Receiver<(ClientId, u64, Connection)>,
    restart: Arc<AtomicBool>,
}
impl SimulationConnection {
    pub fn new(
        simulation_id: SimulationId,
        database_connection: Connection,
        database_response_receiver: Receiver<SimulationResponse>,
        new_client_receiver: Receiver<(ClientId, u64, Connection)>,
        restart: Arc<AtomicBool>,
    ) -> Self {
        Self {
            simulation_id,
            database_connection,
            database_response_receiver,
            new_client_receiver,
            restart,
        }
    }

    pub fn queue(&self, request: SimulationRequest) {
        log::debug!("{:?} -> {:?}", self.simulation_id, request);
        self.database_connection
            .queue(ServerRequest::SimulationRequest {
                simulation_id: self.simulation_id,
                request,
            })
    }

    pub fn try_recv(&self) -> Option<SimulationResponse> {
        if let Ok(response) = self.database_response_receiver.try_recv() {
            log::debug!("{:?} <- {:?}", self.simulation_id, &response);
            Some(response)
        } else {
            None
        }
    }

    pub fn new_client(&self) -> Option<(ClientId, u64, Connection)> {
        if let Ok(new_client) = self.new_client_receiver.try_recv() {
            log::debug!("{:?} <- {:?}", self.simulation_id, new_client.0);
            Some(new_client)
        } else {
            None
        }
    }

    pub fn simulation_id(&self) -> SimulationId {
        self.simulation_id
    }

    pub fn restart(&self) -> bool {
        self.restart.load(std::sync::atomic::Ordering::Relaxed)
    }
}
