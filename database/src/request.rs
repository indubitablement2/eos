use super::*;
use rand::prelude::*;

// ####################################################################################
// ################################### SERVER #########################################
// ####################################################################################

impl Database {
    fn _handle_request(
        &mut self,
        server_id: ServerId,
        request: ServerRequest,
    ) -> Option<ServerResponse> {
        match request {
            ServerRequest::ClientLogin { request, token } => {
                let failure = Some(ServerResponse::ClientLogin {
                    token,
                    result: None,
                });

                if let Some(&client_id) = self.username.get(&request.username) {
                    let client = self.clients.entry(client_id).or_default();

                    let mut hasher = sha2::Sha256::new();
                    hasher.update(request.password.as_bytes());
                    let hash = hasher.finalize();
                    if client.password_sha256.as_deref() != Some(hash.as_slice()) {
                        return failure;
                    }

                    let system_id = *server_id.systems().choose(&mut thread_rng())?;

                    if let Some(prev) = client.connected.replace(system_id) {
                        // Notify previous system of disconnect.
                        self.queue_simulation_response(
                            prev,
                            SimulationResponse::ClientLogoff { client_id },
                        );
                    }

                    Some(ServerResponse::ClientLogin {
                        token,
                        result: Some((client_id, system_id)),
                    })
                } else if request.register {
                    if request.password.len() < 8 {
                        return failure;
                    }

                    let client_id = self.next_client_id;
                    self.next_client_id.next();

                    self.username.insert(request.username, client_id);

                    let mut hasher = sha2::Sha256::new();
                    hasher.update(request.password.as_bytes());
                    let hash = hasher.finalize();

                    let Some(&system_id) = server_id.systems().choose(&mut thread_rng()) else {
                        return failure;
                    };

                    self.clients.insert(
                        client_id,
                        Client {
                            password_sha256: Some(hash.to_vec()),
                            ships: Default::default(),
                            connected: Some(system_id),
                        },
                    );

                    Some(ServerResponse::ClientLogin {
                        token,
                        result: Some((client_id, system_id)),
                    })
                } else {
                    failure
                }
            }
            ServerRequest::PerfStats {} => None,
            ServerRequest::SimulationRequest { system_id, request } => self
                .handle_simulation_request(system_id, request)
                .map(|response| ServerResponse::SimulationResponse {
                    system_id,
                    response,
                }),
        }
    }
}

// ####################################################################################
// ################################### SIMULATION #####################################
// ####################################################################################

impl Database {
    fn handle_simulation_request(
        &mut self,
        system_id: SystemId,
        request: SimulationRequest,
    ) -> Option<SimulationResponse> {
        match request {
            SimulationRequest::ClientLogoff { client_id } => {
                let client = self.clients.get_mut(&client_id)?;

                if client.connected == Some(system_id) {
                    client.connected = None;
                }
                None
            }
        }
    }
}

// ####################################################################################
// ################################### UTIL ###########################################
// ####################################################################################

impl Database {
    pub fn queue_server_response(&self, server_id: ServerId, response: ServerResponse) {
        log::debug!("{:?} <- {:?}", server_id, &response);
        if let Some(server) = &self.servers[server_id] {
            server.connection.queue(response);
        } else {
            log::debug!("Could not send to {:?}. not connected", server_id);
        }
    }

    pub fn queue_simulation_response(&self, system_id: SystemId, response: SimulationResponse) {
        self.queue_server_response(
            system_id.server_id,
            ServerResponse::SimulationResponse {
                system_id,
                response,
            },
        );
    }
}

// ####################################################################################
// ################################### BOILERPLATE ####################################
// ####################################################################################

impl Database {
    pub fn handle_request(&mut self, server_id: ServerId, request: ServerRequest) {
        log::debug!("{:?} -> {:?}", server_id, &request);

        if let Some(response) = self._handle_request(server_id, request) {
            self.queue_server_response(server_id, response);
        }
    }
}
