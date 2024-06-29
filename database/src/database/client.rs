use super::*;

fn hash_password(password: &[u8]) -> [u8; 32] {
    let salt = std::env!("CLIENT_PASSWORD_SALT").as_bytes();
    assert!(!salt.is_empty());

    let mut hasher = sha2::Sha256::new();
    hasher.update(password);
    hasher.update(salt);
    hasher.finalize().into()
}

impl Database {
    pub fn register_client(&mut self, username: String, password: &str) -> Option<ClientId> {
        if password.len() < 6 || password.len() > 32 || username.len() < 4 || username.len() > 32 {
            return None;
        }

        if self.username.contains_key(&username) {
            return None;
        }

        let client_id = self.next_client_id.next();

        let auth_level = if password == database_password() {
            ClientAuthLevel::SuperAdmin
        } else {
            ClientAuthLevel::User
        };

        self.insert_client(
            username,
            client_id,
            hash_password(password.as_bytes()),
            auth_level,
        );

        log::debug!("{:?} registered", client_id);
        Some(client_id)
    }

    pub fn insert_client(
        &mut self,
        username: String,
        client_id: ClientId,
        password_sha256: [u8; 32],
        auth_level: ClientAuthLevel,
    ) {
        self.username.insert(username.clone(), client_id);
        let client = Client {
            auth_level,
            username,
            password_sha256,
            ships: Default::default(),
            connection: None,
            simulation: None,
            connection_generation: 0,
        };
        self.saver
            .as_ref()
            .unwrap()
            .insert_client(client_id, &client);
        self.clients.insert(client_id, client);
    }

    pub fn connect_client(
        &mut self,
        client_id: ClientId,
        password: &str,
        connection: Connection,
    ) -> Option<()> {
        let client = self.clients.get(&client_id)?;

        if client.password_sha256 != hash_password(password.as_bytes()) {
            return None;
        }

        self.disconnect_client(client_id);

        connection.queue(client_id);

        let client = self.clients.get_mut(&client_id)?;
        client.connection = Some(connection.clone());
        self.client_connections
            .push((client_id, connection, client.connection_generation));

        log::debug!("{:?} connected", client_id);
        Some(())
    }

    pub fn disconnect_client(&mut self, client_id: ClientId) -> Option<Connection> {
        let client = self.clients.get_mut(&client_id)?;
        let connection = client.connection.take()?;

        client.connection_generation += 1;

        if let Some(simulation_id) = client.simulation.take() {
            self.disconnect_client_from_simulation(client_id, simulation_id);
        }

        log::debug!("{:?} disconnected", client_id);
        Some(connection)
    }

    pub fn disconnect_client_from_simulation(
        &mut self,
        client_id: ClientId,
        simulation_id: SimulationId,
    ) -> Option<()> {
        let client = self.clients.get_mut(&client_id)?;
        if client.simulation != Some(simulation_id) {
            return None;
        }
        client.simulation = None;

        let simulation = self.simulations.get_mut(&simulation_id)?;
        simulation.connected_clients.remove(&client_id);

        let server = self.servers.get_mut(&simulation.handling_server)?;
        server.connection.queue(ServerResponse::SimulationResponse {
            simulation_id,
            response: SimulationResponse::ClientAuthorisationRemove { client_id },
        });

        log::debug!("{:?} disconnected from {:?}", client_id, simulation_id);
        Some(())
    }
}
