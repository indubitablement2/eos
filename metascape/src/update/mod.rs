mod apply_fleets_movement;
mod handle_fleet_queue;
mod update_fleets_detected_acc;
mod update_fleets_in_system;

use self::apply_fleets_movement::*;
use self::handle_fleet_queue::*;
use self::update_fleets_detected_acc::*;
use self::update_fleets_in_system::*;
use super::*;

pub type NewFleetQueue = Vec<FleetBuilder>;

impl Metascape {
    pub fn update_internal(&mut self) {
        let mut new_fleet_queue: NewFleetQueue = Default::default();

        // TODO: Remove this.
        // Populate with a bunch of random fleet.
        if self.tick == 5 {
            let orbit_time = orbit_time(self.tick);

            for system in self.systems.systems.values() {
                for planet in system.planets.iter() {
                    let position = planet
                        .relative_orbit
                        .to_position(orbit_time, system.position)
                        + rand_vec2(&mut self.rng, -10.0..10.0);

                    new_fleet_queue.push(FleetBuilder::new(
                        self.fleet_id_dispenser.next(),
                        position,
                        common::data()
                            .starting_fleets
                            .choose(&mut self.rng)
                            .unwrap()
                            .to_owned(),
                    ));
                }
            }
        }

        // TODO: Faction ai.

        handle_fleet_queue(new_fleet_queue, &mut self.fleets, &mut self.clients, &mut self.factions);

        // TODO: AutoCombat
        // TODO: Collision

        // TODO: Fleet ai.

        apply_fleets_movement(self);

        update_fleets_in_system(self);

        update_fleets_detected_acc(self);

        self.tick += 1;
    }
}
