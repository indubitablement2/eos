use super::*;

impl Database {
    pub fn insert_simulation(&mut self, simulation_id: SimulationId, position: Vec2, zone: u64) {
        let handling_servers = self
            .server_zones
            .get(&zone)
            .unwrap_or_else(|| self.server_zones.values().next().unwrap());
        let asd = handling_servers
            .iter()
            .map(|server_id| (*server_id, self.servers.get_mut(server_id).unwrap()))
            .min_by_key(|(server_id, server)| server.current_simulation_cost)
            .unwrap();

        let simulation = Simulation {
            handling_server: todo!(),
            connected_clients: Default::default(),
            position,
            zone,
            ships: Default::default(),
        };

        // let mut simulation = self.simulations.remove(&simulation_id)?;
        // if let Some(server) = self.servers.get_mut(&simulation.handling_server) {
        //     server.simulations.remove(&simulation_id);
        // }
        // self.free_simulation.insert(*simulation_id);

        // for client_id in simulation.connected_clients.drain() {
        //     if let Some(client) = self.clients.get_mut(&client_id) {
        //         client.simulation = None;
        //     }
        // }
    }
}
