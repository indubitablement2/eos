mod apply_fleets_movement;
mod connect_clients;
mod handle_clients_inputs;
mod handle_faction_queue;
mod handle_fleet_queue;
mod send_detected_entities;
mod update_fleets_detected_acc;
mod update_fleets_in_system;

use self::apply_fleets_movement::*;
use self::connect_clients::*;
use self::handle_clients_inputs::*;
use self::handle_faction_queue::*;
use self::handle_fleet_queue::*;
use self::send_detected_entities::*;
use self::update_fleets_detected_acc::*;
use self::update_fleets_in_system::*;
use super::*;

impl Metascape {
    pub fn update_internal(&mut self) {
        unsafe {
            _TICK += 1;
            _TOTAL_TICK += 1;
        }

        connect_clients(self);

        let disconnected = handle_clients_inputs(self);
        // Remove disconnected clients.
        for client_id in disconnected.into_iter() {
            self.clients
                .swap_remove_by_id(client_id)
                .expect("There should be a client");
            log::debug!("Removed {:?} from metascape.", client_id)
        }

        handle_faction_queue(self);

        handle_fleet_queue(self);

        // No more add/remove fleet from this point.

        // No more change to fleet's composition from this point.

        // TODO: AI

        apply_fleets_movement(self);

        update_fleets_in_system(self);

        update_fleets_detected_acc(self);

        // TODO: send factions infos the client request.
        send_detected_entities(self);
    }
}
