use super::*;

impl Database {
    pub fn create_ship(
        &mut self,
        ship_data_id: ShipDataId,
        simulation_id: SimulationId,
        owner: Option<ClientId>,
        position: Vec2,
    ) -> Option<ShipId> {
        let ship_id = self.next_ship_id.next();
        self.insert_ship(ship_id, ship_data_id, simulation_id, owner, position)
    }

    pub fn insert_ship(
        &mut self,
        ship_id: ShipId,
        ship_data_id: ShipDataId,
        simulation_id: SimulationId,
        mut owner: Option<ClientId>,
        position: Vec2,
    ) -> Option<ShipId> {
        let simulation = self.simulations.get_mut(&simulation_id)?;

        if let Some(client_id) = owner.take() {
            if let Some(client) = self.clients.get_mut(&client_id) {
                client.ships.insert(ship_id);
                owner = Some(client_id);
            } else {
                log::warn!("{:?} does not exist", client_id);
            }
        }

        let ship = Ship {
            ship_data_id,
            owner,
            simulation_id,
            position,
        };
        self.saver.as_ref().unwrap().insert_ship(ship_id, &ship);

        self.ships.insert(ship_id, ship);

        simulation.ships.insert(ship_id);
        if let Some(server) = self.servers.get(&simulation.handling_server) {
            server.connection.queue(ServerResponse::SimulationResponse {
                simulation_id,
                response: SimulationResponse::ShipEnter {
                    ship_id,
                    ship_data_id,
                    position,
                },
            });
        }

        Some(ship_id)
    }

    pub fn save_ship(
        &mut self,
        simulation_id: SimulationId,
        ship_id: ShipId,
        position: Vec2,
    ) -> Option<()> {
        let ship = self.ships.get_mut(&ship_id)?;
        if ship.simulation_id != simulation_id {
            // TODO: Notify simulation that it doesn't have this ship.
            return None;
        }
        ship.position = position;
        self.saver.as_ref().unwrap().insert_ship(ship_id, &ship);
        Some(())
    }

    pub fn move_ship() {
        // todo
    }
}
