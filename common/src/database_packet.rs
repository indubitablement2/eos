use self::{connection::Connection, ids::*, server::ServerId, system::SystemId};
use super::*;
use flume::Receiver;

pub struct SimulationConnection {
    system_id: SystemId,
    connection: Connection,
    receiver: Receiver<SimulationResponse>,
    new_client: Receiver<(ClientId, Connection)>,
}
impl SimulationConnection {
    pub fn new(
        system_id: SystemId,
        connection: Connection,
        receiver: Receiver<SimulationResponse>,
        new_client: Receiver<(ClientId, Connection)>,
    ) -> Self {
        Self {
            system_id,
            connection,
            receiver,
            new_client,
        }
    }

    pub fn queue(&self, request: SimulationRequest) {
        self.connection.queue(ServerRequest::SimulationRequest {
            system_id: self.system_id,
            request,
        })
    }

    pub fn try_recv(&self) -> Option<SimulationResponse> {
        self.receiver.try_recv().ok()
    }

    pub fn new_client(&self) -> Option<(ClientId, Connection)> {
        self.new_client.try_recv().ok()
    }

    pub fn system_id(&self) -> SystemId {
        self.system_id
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerAuthRequest {
    pub server_id: ServerId,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerAuthResponse {
    pub password: String,
    pub system_saves: Vec<(SystemId, Option<Vec<u8>>)>,
}

#[derive(Serialize, Deserialize)]
pub enum ServerRequest {
    ClientLogin {
        username: String,
        password: String,
        token: u64,
    },
    ClientRegister {
        username: String,
        password: String,
        token: u64,
    },
    PerfStats {},
    SimulationRequest {
        system_id: SystemId,
        request: SimulationRequest,
    },
}

#[derive(Serialize, Deserialize)]
pub enum ServerResponse {
    ClientLogin {
        token: u64,
        client_id: Option<ClientId>,
        fail_reason: Option<String>,
    },
    SimulationResponse {
        system_id: SystemId,
        request: SimulationResponse,
    },
}

#[derive(Serialize, Deserialize)]
pub enum SimulationRequest {}

#[derive(Serialize, Deserialize)]
pub enum SimulationResponse {
    ClientUpdate {
        client_id: ClientId,
        update: ClientUpdate,
    },
}

#[derive(Serialize, Deserialize)]
pub struct ClientUpdate {
    pub ships_delta: Vec<()>,
}
