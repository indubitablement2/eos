use super::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub enum ClientRequest {}

#[derive(Serialize)]
pub enum ClientResponse {}

impl Database {
    pub fn handle_client_request(&mut self, client_id: ClientId, request: ClientRequest) {
        log::debug!("{:?} -> {:?}", client_id, &request);

        match request {
            // ClientRequest::PerfStats {} => {}
        }
    }
}
