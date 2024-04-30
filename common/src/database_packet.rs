use self::ids::*;
use super::*;
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

#[derive(Serialize, Deserialize)]
pub enum ServerRequest {}

#[derive(Serialize, Deserialize)]
pub enum ServerResponse {}

#[derive(Deserialize)]
pub enum ClientRequest {}

#[derive(Serialize)]
pub enum ClientResponse {
    /// Someones else logged in with the same account.
    MultipleLogin,
}
