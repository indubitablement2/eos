use super::*;

impl Battlescape {
    pub fn apply_commands(&mut self, cmds: &[BattlescapeCommand]) {
        for cmd in cmds {
            match cmd {
                BattlescapeCommand::AddFleet(add_fleet) => {
                    let battlescape_fleet = BattlescapeFleet {
                        original_fleet: add_fleet.fleet.clone(),
                        available_ships: AHashMap::from_iter(
                            (0..add_fleet.fleet.ships.len())
                                .map(|index| (index, self.new_ship_id())),
                        ),
                        team: add_fleet.team.unwrap_or_else(|| self.new_team()),
                    };

                    self.fleets.insert(add_fleet.fleet_id, battlescape_fleet);

                    log::debug!("Added {:?}", add_fleet.fleet_id);
                }
            }
        }
    }
}
