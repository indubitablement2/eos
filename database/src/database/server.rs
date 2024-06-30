use super::*;

impl Database {
    pub fn insert_server(
        &mut self,
        given_database_password: String,
        server_address: String,
        simulation_capacity: f32,
        connection: Connection,
    ) -> Option<ServerId> {
        if given_database_password != database_password() {
            return None;
        }

        let zone = server_address
            .get(..2)?
            .as_bytes()
            .iter()
            .enumerate()
            .fold(0u64, |acc, (idx, byte)| acc | ((*byte as u64) << (idx * 8)));

        let server_id = self.next_sever_id.next();
        log::info!("Server authenticated: {}, {}", server_address, zone);

        connection.queue(server_id);

        let connection_rand_generation = random();

        self.servers.insert(
            server_id,
            Server {
                connection: connection.clone(),
                connection_rand_generation,
                zone,
                server_address,
                simulations: Default::default(),
                simulation_capacity,
                current_simulation_cost: 0.0,
            },
        );
        self.server_connections
            .push((server_id, connection, connection_rand_generation));
        self.server_zones.entry(zone).or_default().push(server_id);

        Some(server_id)
    }

    pub fn remove_server(&mut self, server_id: ServerId) -> Option<Server> {
        let server = self.servers.remove(&server_id)?;

        if let Some(server_zones) = self.server_zones.get_mut(&server.zone) {
            if let Some(idx) = server_zones.iter().position(|&id| id == server_id) {
                server_zones.swap_remove(idx);
            }
            if server_zones.is_empty() {
                self.server_zones.remove(&server.zone);
            }
        }

        for simulation_id in &server.simulations {
            if let Some(simulation) = self.simulations.get(simulation_id) {
                for client_id in simulation.connected_clients.iter() {
                    if let Some(client) = self.clients.get_mut(&client_id) {
                        client.simulation = None;
                    }
                }

                self.insert_simulation(*simulation_id, simulation.position, simulation.zone);
            }
        }

        log::warn!("{:?} connection closed", server_id);
        Some(server)
    }
}
