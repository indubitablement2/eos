use self::ids::*;
use super::*;
use bitcode::{Decode, Encode};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
pub enum AuthRequest {
    ClientLogin {
        username: String,
        password: String,
    },
    ClientRegister {
        username: String,
        password: String,
    },
    Server {
        password: String,
    },
    Simulation {
        password: String,
        server_id: ServerId,
        client_address: SocketAddr,
    },
}

#[derive(Encode, Decode)]
pub enum ServerRequest {}

#[derive(Encode, Decode)]
pub enum ServerResponse {}

#[derive(Encode, Decode)]
pub enum SimulationRequest {}

#[derive(Encode, Decode)]
pub enum SimulationResponse {}

#[derive(Deserialize)]
pub enum ClientRequest {}

#[derive(Serialize)]
pub enum ClientResponse {
    /// Someones else logged in with the same account.
    MultipleLogin,
}
