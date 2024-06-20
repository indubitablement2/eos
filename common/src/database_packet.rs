use self::{client_data::*, connection::Connection, ids::*};
use super::*;
use flume::{unbounded, Receiver, Sender};
use redis::Commands;

#[derive(Serialize, Deserialize)]
pub struct ServerAuthRequest {
    pub password: Vec<u8>,
    // TODO: Client address?
}

#[derive(Serialize, Deserialize)]
pub struct ServerAuthResponse {
    pub server_id: ServerId,
}

// ####################################################################################
// ################################### SERVER #########################################
// ####################################################################################

/// Server -> Database
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerRequest {
    SimulationRequest {
        simulation_id: (),
        request: SimulationRequest,
    },
    ClientLogin {
        client_login: ClientLogin,
        token: u64,
    },
}

/// Database -> Server
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerResponse {
    SimulationResponse {
        simulation_id: (),
        response: SimulationResponse,
    },
    ClientLoginSuccess {
        client_id: ClientId,
        login_token: u64,
        client_data: ClientData,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientLogin {
    pub username: String,
    pub password: String,
    /// Try to register if the username does not exist.
    /// Otherwise try to login.
    pub register: bool,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct ClientJoinWithCharacter {}

// ####################################################################################
// ################################### SIMULATION #####################################
// ####################################################################################

/// Simulation -> Database
#[derive(Debug, Serialize, Deserialize)]
pub enum SimulationRequest {
    ClientLogoff { client_id: ClientId },
    Save { data: Vec<u8> },
}

/// Database -> Simulation
#[derive(Debug, Serialize, Deserialize)]
pub enum SimulationResponse {
    // NewSimulation{...},
    ClientLeft { client_id: ClientId },
}

pub struct SimulationConnection {
    simulation_id: (),
    /// Used to send requests only.
    database_connection: Connection,
    database_response_receiver: Receiver<SimulationResponse>,
    client_receiver: Receiver<(
        ClientId,
        Connection,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
    )>,
}
impl SimulationConnection {
    pub fn new(
        simulation_id: (),
        database_connection: Connection,
    ) -> (
        Self,
        Sender<SimulationResponse>,
        Sender<(
            ClientId,
            Connection,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
        )>,
    ) {
        let (database_response_sender, database_response_receiver) = unbounded();
        let (client_sender, client_receiver) = unbounded();

        (
            Self {
                simulation_id,
                database_connection,
                database_response_receiver,
                client_receiver,
            },
            database_response_sender,
            client_sender,
        )
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

    pub fn try_recv_client(
        &self,
    ) -> Option<(
        ClientId,
        Connection,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
    )> {
        if let Ok(response) = self.client_receiver.try_recv() {
            log::debug!("{:?} joined {:?}", response.0, self.simulation_id);
            Some(response)
        } else {
            None
        }
    }
}
