use super::*;
use common::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub enum ClientRequest {}

#[derive(Serialize)]
pub enum ClientResponse {
    InsertSimulation {
        simulation_id: SimulationId,
        zone: u64,
        position: Vec2,
    },
}

impl Database {
    pub fn handle_client_request(&mut self, client_id: ClientId, request: ClientRequest) {
        log::debug!("{:?} -> {:?}", client_id, &request);

        match request {}
    }
}
