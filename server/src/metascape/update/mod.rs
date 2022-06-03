mod apply_fleets_movement;
mod connect_clients;
mod handle_clients_inputs;
mod send_detected_entities;
mod update_fleets_acc;
mod update_fleets_in_system;
mod handle_fleet_queue;
mod handle_changed_fleet;
mod handle_faction_queue;

use super::*;
use self::apply_fleets_movement::*;
use self::connect_clients::*;
use self::handle_clients_inputs::*;
use self::send_detected_entities::*;
use self::update_fleets_acc::*;
use self::update_fleets_in_system::*;
use self::handle_fleet_queue::*;
use self::handle_changed_fleet::*;
use self::handle_faction_queue::*;

impl Metascape {
    pub fn update_internal(&mut self) {
        unsafe {
            _TIME.tick += 1;
            _TIME.total_tick += 1;
        }

        connect_clients(self);

        handle_faction_queue(self);
        handle_fleet_queue(self);

        handle_clients_inputs(self);

        // TODO: AI

        // No more change to fleet's composition from this point.

        handle_changed_fleet(self);

        apply_fleets_movement(self);

        update_fleets_detection_acceleration_structure(self);
        update_fleets_in_system(self);

        // TODO: send factions infos the client request.
        send_detected_entities(self);
    }
}
