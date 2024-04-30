use super::*;

pub enum Mutation {
    SimulationAuth {
        connection: Connection,
        server_id: ServerId,
        client_address: SocketAddr,
    },
    ServerAuth {
        connection_idx: usize,
    },
    ClientLogin {
        connection_idx: usize,
        client_id: ClientId,
    },
    ClientRegister {
        connection_idx: usize,
        username: String,
        password: String,
    },
}

impl Database {
    pub fn handle_auth_request(
        &self,
        request: AuthRequest,
        connection_idx: usize,
        connection: &Connection,
    ) -> Option<Mutation> {
        match request {
            AuthRequest::ClientLogin { username, password } => {
                let client_id = self.username.get(&username)?;
                let client = self.clients.get(client_id)?;

                let mut hasher = sha2::Sha256::new();
                hasher.update(&client.password_salt);
                hasher.update(password);
                let hash = hasher.finalize();
                if client.password_sha256.as_deref() != Some(hash.as_slice()) {
                    return None;
                }

                Some(Mutation::ClientLogin {
                    connection_idx,
                    client_id: *client_id,
                })
            }
            AuthRequest::ClientRegister { username, password } => {
                if self.username.contains_key(&username) {
                    return None;
                }

                if password.len() < 8 {
                    return None;
                }

                Some(Mutation::ClientRegister {
                    connection_idx,
                    username,
                    password,
                })
            }
            AuthRequest::Server { password } => {
                if password != self.password {
                    return None;
                }

                Some(Mutation::ServerAuth { connection_idx })
            }
            AuthRequest::Simulation {
                password,
                server_id,
                client_address,
            } => {
                if password != self.password {
                    return None;
                }

                Some(Mutation::SimulationAuth {
                    connection: connection.clone(),
                    server_id,
                    client_address,
                })
            }
        }
    }

    pub fn apply_auth_mutation(&mut self, mutation: Mutation) -> Option<()> {
        match mutation {
            Mutation::SimulationAuth {
                connection,
                server_id,
                client_address,
            } => {
                self.servers
                    .get_mut(&server_id)?
                    .stand_by_runner
                    .push(SystemRunner {
                        connection,
                        client_address,
                    });
            }
            Mutation::ServerAuth { connection_idx } => {
                let (connection_type, connection) = self.connections.get_mut(connection_idx)?;

                let server_id = self.next_server_id;
                self.next_server_id.next();

                *connection_type = ConnectionType::Server(server_id);
                connection.queue(server_id);
                self.servers.insert(
                    server_id,
                    Server {
                        connection: connection.clone(),
                        saturation: 100,
                        stand_by_runner: Default::default(),
                        systems: Default::default(),
                    },
                );
            }
            Mutation::ClientLogin {
                connection_idx,
                client_id,
            } => {
                let (connection_type, connection) = self.connections.get_mut(connection_idx)?;

                let client = self.clients.get_mut(&client_id)?;

                if let Some(prev) = client.connection.take() {
                    prev.queue(ClientResponse::MultipleLogin);
                    prev.close();

                    return Some(());
                }

                *connection_type = ConnectionType::Client(client_id);
                connection.queue(client_id);
                client.connection = Some(connection.clone());
            }
            Mutation::ClientRegister {
                connection_idx,
                username,
                password,
            } => {
                let (connection_type, connection) = self.connections.get_mut(connection_idx)?;

                if self.username.contains_key(&username) {
                    return Some(());
                }

                let client_id = self.next_client_id;
                self.next_client_id.next();

                self.username.insert(username, client_id);

                let password_salt: [u8; 8] = rand::random();

                let mut hasher = sha2::Sha256::new();
                hasher.update(&password_salt);
                hasher.update(password);
                let hash = hasher.finalize();

                *connection_type = ConnectionType::Client(client_id);
                connection.queue(client_id);
                self.clients.insert(
                    client_id,
                    Client {
                        password_salt,
                        password_sha256: Some(hash.to_vec()),
                        ships: Default::default(),
                        connection: Some(connection.clone()),
                    },
                );
            }
        }

        Some(())
    }
}
