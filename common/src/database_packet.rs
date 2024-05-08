use std::sync::{atomic::AtomicBool, Arc};

use self::{connection::Connection, ids::*, server::ServerId, system::SystemId};
use super::*;
use flume::Receiver;

// ####################################################################################
// ################################### SERVER AUTH ####################################
// ####################################################################################

#[derive(Serialize, Deserialize)]
pub struct ServerAuthRequest {
    pub server_id: ServerId,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerAuthResponse {
    pub system_saves: Vec<(SystemId, Option<Vec<u8>>)>,
}

// ####################################################################################
// ################################### SERVER #########################################
// ####################################################################################

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerRequest {
    ClientLogin {
        request: ClientLogin,
        token: u64,
    },
    PerfStats {},
    SimulationRequest {
        system_id: SystemId,
        request: SimulationRequest,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientLogin {
    /// None -> join any system handled by this server.
    pub join_system: Option<SystemId>,
    pub username: String,
    pub password: String,
    /// Try to register if the username does not exist.
    /// Otherwise try to login.
    pub register: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerResponse {
    ClientLogin {
        token: u64,
        result: Option<(ClientId, SystemId)>,
    },
    SimulationResponse {
        system_id: SystemId,
        response: SimulationResponse,
    },
    Restart,
}

// ####################################################################################
// ################################### SIMULATION #####################################
// ####################################################################################

#[derive(Debug, Serialize, Deserialize)]
pub enum SimulationRequest {
    ClientLogoff { client_id: ClientId },
    Save { save: Vec<u8> },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SimulationResponse {
    ClientUpdate {
        client_id: ClientId,
        update: ClientUpdate,
    },
    ClientLogoff {
        client_id: ClientId,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientUpdate {
    pub ships_delta: Vec<()>,
}

pub struct SimulationConnection {
    system_id: SystemId,
    /// Used to send requests only.
    database_connection: Connection,
    database_response_receiver: Receiver<SimulationResponse>,
    new_client_receiver: Receiver<(ClientId, Connection)>,
    restart: Arc<AtomicBool>,
}
impl SimulationConnection {
    pub fn new(
        system_id: SystemId,
        database_connection: Connection,
        database_response_receiver: Receiver<SimulationResponse>,
        new_client_receiver: Receiver<(ClientId, Connection)>,
        restart: Arc<AtomicBool>,
    ) -> Self {
        Self {
            system_id,
            database_connection,
            database_response_receiver,
            new_client_receiver,
            restart,
        }
    }

    pub fn queue(&self, request: SimulationRequest) {
        log::debug!("{:?} -> {:?}", self.system_id, request);
        self.database_connection
            .queue(ServerRequest::SimulationRequest {
                system_id: self.system_id,
                request,
            })
    }

    pub fn try_recv(&self) -> Option<SimulationResponse> {
        if let Ok(response) = self.database_response_receiver.try_recv() {
            log::debug!("{:?} <- {:?}", self.system_id, &response);
            Some(response)
        } else {
            None
        }
    }

    pub fn new_client(&self) -> Option<(ClientId, Connection)> {
        if let Ok(new_client) = self.new_client_receiver.try_recv() {
            log::debug!("{:?} <- {:?}", self.system_id, new_client.0);
            Some(new_client)
        } else {
            None
        }
    }

    pub fn system_id(&self) -> SystemId {
        self.system_id
    }

    pub fn restart(&self) -> bool {
        self.restart.load(std::sync::atomic::Ordering::Relaxed)
    }
}
