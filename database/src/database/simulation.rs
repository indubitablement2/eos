use super::*;

impl Database {
    pub fn insert_simulation(&mut self, simulation_id: SimulationId, simulation: Simulation) {
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
