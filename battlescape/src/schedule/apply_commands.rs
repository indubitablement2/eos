use super::*;
use commands::*;

impl Battlescape {
    pub fn apply_commands(&mut self, cmds: &[BattlescapeCommand]) {
        for cmd in cmds {
            match cmd {
                BattlescapeCommand::AddFleet(cmd) => self.add_fleet(cmd),
                BattlescapeCommand::SetClientControl(cmd) => self.set_client_control(cmd),
                BattlescapeCommand::SetClientInput(cmd) => self.set_client_input(cmd),
            }
        }
    }

    fn add_fleet(&mut self, cmd: &AddFleet) {
        let battlescape_fleet = BattlescapeFleet {
            original_fleet: cmd.fleet.clone(),
            available_ships: AHashMap::from_iter(
                (0..cmd.fleet.ships.len()).map(|index| (index, self.new_ship_id())),
            ),
            team: cmd.team.unwrap_or_else(|| self.new_team()),
        };

        if self
            .fleets
            .insert(cmd.fleet_id, battlescape_fleet)
            .is_some()
        {
            log::warn!("Overwritten {:?}", cmd.fleet_id);
        }
        log::info!(
            "Added {:?} with {} ships",
            cmd.fleet_id,
            cmd.fleet.ships.len()
        );
    }

    fn set_client_control(&mut self, cmd: &SetClientControl) {
        let client = self.clients.entry(cmd.client_id).or_default();
        client.last_active = self.tick;
        client.control = None;

        if let Some(ship_id) = cmd.ship_id {
            // Check that client own that ship.
            if let Some(ship) = self.ships.get_mut(&ship_id) {
                if self
                    .fleets
                    .get(&ship.fleet_id)
                    .unwrap()
                    .original_fleet
                    .owner
                    == Some(cmd.client_id)
                {
                    client.control = Some(ship_id);
                    ship.contol = Some(cmd.client_id);
                } else {
                    log::debug!(
                        "{:?} does not own {:?}. Removing control",
                        cmd.client_id,
                        cmd.ship_id
                    );
                }
            } else {
                log::debug!("{:?} does not exist. Removing control", cmd.ship_id);
            }
        }
    }

    fn set_client_input(&mut self, cmd: &SetClientInput) {
        let client = self.clients.entry(cmd.client_id).or_default();
        client.last_active = self.tick;
        client.last_inputs = cmd.inputs.validate();
    }
}
