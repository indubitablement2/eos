mod apply_fleets_movement;
mod connect_clients;
mod handle_clients_inputs;
mod handle_disconnect;
mod handle_fleet_queue;
mod send_detected_entities;
mod update_fleets_detected_acc;
mod update_fleets_in_system;
mod update_masks;

use common::timef::TimeF;

use self::apply_fleets_movement::*;
use self::connect_clients::*;
use self::handle_clients_inputs::*;
use self::handle_disconnect::*;
use self::handle_fleet_queue::*;
use self::send_detected_entities::*;
use self::update_fleets_detected_acc::*;
use self::update_fleets_in_system::*;
use self::update_masks::*;
use super::*;

pub type NewFleetQueue = Vec<FleetBuilder>;

impl<C> Metascape<C>
where
    C: ConnectionsManager,
{
    pub fn update_internal(&mut self, connections_manager: &mut C) {
        let mut disconnect: Vec<(C::ConnectionType, Option<DisconnectedReason>)> =
            Default::default();
        let mut new_fleet_queue: NewFleetQueue = Default::default();

        self.tick += 1;
        self.total_tick += 1;

        // TODO: Remove this.
        // Populate with a bunch of random fleet.
        if self.total_tick == 5 {
            for system in self.systems.systems.values() {
                for planet in system.planets.iter() {
                    let position = planet
                        .relative_orbit
                        .to_position(TimeF::tick_to_orbit_time(self.total_tick), system.position)
                        + Vec2 { x: self.rng.gen_range(-10.0..10.0), y: self.rng.gen_range(-10.0..10.0) };
                    
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

        connect_clients(self, connections_manager, &mut disconnect);
        handle_disconnect(connections_manager, &mut disconnect);

        handle_clients_inputs(self, &mut disconnect, &mut new_fleet_queue);
        handle_disconnect(connections_manager, &mut disconnect);

        // TODO: Faction ai.

        handle_fleet_queue(self, new_fleet_queue);

        // TODO: AutoCombat
        // TODO: Collision

        // TODO: Fleet ai.

        update_masks(self);
        apply_fleets_movement(self);

        update_fleets_in_system(self);

        update_fleets_detected_acc(self);

        send_detected_entities(self, &mut disconnect);
        handle_disconnect(connections_manager, &mut disconnect);
    }
}
