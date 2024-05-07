use super::*;
use rand::prelude::*;

// ####################################################################################
// ################################### SERVER #########################################
// ####################################################################################

pub enum ServerMutation {
    ClientLogin {
        token: u64,
        client_id: ClientId,
    },
    ClientRegister {
        token: u64,
        username: String,
        password: String,
    },
}

impl Database {
    fn _handle_request(&self, server_id: ServerId, request: ServerRequest) -> Option<Mutation> {
        let server_mutation = match request {
            ServerRequest::ClientLogin { request, token } => {
                if let Some(client_id) = self.username.get(&request.username) {
                    let client = self.clients.get(client_id)?;

                    let mut hasher = sha2::Sha256::new();
                    hasher.update(&client.password_salt);
                    hasher.update(request.password.as_bytes());
                    let hash = hasher.finalize();
                    if client.password_sha256.as_deref() != Some(hash.as_slice()) {
                        return None;
                    }

                    Some(ServerMutation::ClientLogin {
                        token,
                        client_id: *client_id,
                    })
                } else if request.register {
                    if request.password.len() < 8 {
                        return None;
                    }

                    Some(ServerMutation::ClientRegister {
                        token,
                        username: request.username,
                        password: request.password,
                    })
                } else {
                    None
                }
            }
            ServerRequest::PerfStats {} => None,
            ServerRequest::SimulationRequest { system_id, request } => {
                return self
                    .handle_simulation_request(system_id, request)
                    .map(|mutation| Mutation::SimulationMutation(system_id, mutation));
            }
        };

        server_mutation.map(|mutation| Mutation::ServerMutation(server_id, mutation))
    }

    fn apply_server_mutation(
        &mut self,
        server_id: ServerId,
        mutation: ServerMutation,
    ) -> Option<ServerResponse> {
        match mutation {
            ServerMutation::ClientLogin { token, client_id } => {
                let system_id = *server_id.systems().choose(&mut thread_rng())?;

                if let Some(prev) = self
                    .clients
                    .get_mut(&client_id)?
                    .connected
                    .replace(system_id)
                {
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
            }
            ServerMutation::ClientRegister {
                token,
                username,
                password,
            } => {
                let failure = Some(ServerResponse::ClientLogin {
                    token,
                    result: None,
                });

                if self.username.contains_key(&username) {
                    return failure;
                }

                let client_id = self.next_client_id;
                self.next_client_id.next();

                self.username.insert(username, client_id);

                let password_salt: [u8; 8] = rand::random();

                let mut hasher = sha2::Sha256::new();
                hasher.update(&password_salt);
                hasher.update(password.as_bytes());
                let hash = hasher.finalize();

                self.clients.insert(
                    client_id,
                    Client {
                        password_salt,
                        password_sha256: Some(hash.to_vec()),
                        ships: Default::default(),
                        connected: None,
                    },
                );

                self.apply_server_mutation(
                    server_id,
                    ServerMutation::ClientLogin { token, client_id },
                )
            }
        }
    }
}

// ####################################################################################
// ################################### SIMULATION #####################################
// ####################################################################################

pub enum SimulationMutation {
    ClientLogoff { client_id: ClientId },
}

impl Database {
    fn handle_simulation_request(
        &self,
        system_id: SystemId,
        request: SimulationRequest,
    ) -> Option<SimulationMutation> {
        match request {
            SimulationRequest::ClientLogoff { client_id } => {
                Some(SimulationMutation::ClientLogoff { client_id })
            }
        }
    }

    fn apply_simulation_mutation(
        &mut self,
        system_id: SystemId,
        mutation: SimulationMutation,
    ) -> Option<SimulationResponse> {
        match mutation {
            SimulationMutation::ClientLogoff { client_id } => {
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
        if let Some(server) = &self.servers[server_id] {
            server.connection.queue(response);
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

pub enum Mutation {
    ServerMutation(ServerId, ServerMutation),
    SimulationMutation(SystemId, SimulationMutation),
}

impl Database {
    pub fn handle_request(&self, server_id: ServerId, request: ServerRequest) {
        if let Some(mutation) = self._handle_request(server_id, request) {
            self.mutations.get_or_default().borrow_mut().push(mutation);
        }
    }

    pub fn apply_mutation(&mut self, mutation: Mutation) {
        match mutation {
            Mutation::ServerMutation(server_id, mutation) => {
                if let Some(response) = self.apply_server_mutation(server_id, mutation) {
                    self.queue_server_response(server_id, response);
                }
            }
            Mutation::SimulationMutation(system_id, mutation) => {
                if let Some(response) = self.apply_simulation_mutation(system_id, mutation) {
                    self.queue_simulation_response(system_id, response);
                }
            }
        }
    }
}
