use super::*;

impl Database {
    pub fn handle_server_request(&mut self, server_id: ServerId, request: ServerRequest) {
        log::debug!("{:?} -> {:?}", server_id, &request);

        match request {
            ServerRequest::PerfStats {} => {}
            ServerRequest::SimulationRequest {
                simulation_id,
                request,
            } => self.handle_simulation_request(simulation_id, request),
        }
    }
}

// ####################################################################################
// ################################### SIMULATION #####################################
// ####################################################################################

impl Database {
    fn handle_simulation_request(
        &mut self,
        simulation_id: SimulationId,
        request: SimulationRequest,
    ) {
        match request {
            SimulationRequest::ClientLogoff { client_id } => {
                self.disconnect_client_from_simulation(client_id, simulation_id);
            }
            SimulationRequest::Save { ship_saves } => {
                for (ship_id, position) in ship_saves {
                    self.save_ship(simulation_id, ship_id, position);
                }
            }
            SimulationRequest::SaveShip { ship_id, position } => {
                self.save_ship(simulation_id, ship_id, position);
            }
            SimulationRequest::CreateShip {
                ship_data_id,
                owner,
                position,
            } => {
                self.create_ship(ship_data_id, simulation_id, owner, position);
            }
        }
    }
}
