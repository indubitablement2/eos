use super::*;

impl Database {
    pub fn create_ship(
        &mut self,
        simulation_id: SimulationId,
        ship_data_id: ShipDataId,
        position: Vec2,
    ) -> Option<ShipId> {
        let ship_id = self.next_ship_id.next();
        self.insert_ship(ship_id, simulation_id, ship_data_id, position)
    }

    pub fn insert_ship(
        &mut self,
        ship_id: ShipId,
        simulation_id: SimulationId,
        ship_data_id: ShipDataId,
        position: Vec2,
    ) -> Option<ShipId> {
        let simulation = self.simulations.get_mut(&simulation_id)?;

        let ship = Ship {
            ship_data_id,
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
