mod apply_fleets_movement;
mod connect_clients;
mod handle_clients_inputs;
mod handle_disconnect;
mod handle_faction_queue;
mod handle_fleet_queue;
mod send_detected_entities;
mod update_fleets_detected_acc;
mod update_fleets_in_system;

use self::apply_fleets_movement::*;
use self::connect_clients::*;
use self::handle_clients_inputs::*;
use self::handle_disconnect::*;
use self::handle_faction_queue::*;
use self::handle_fleet_queue::*;
use self::send_detected_entities::*;
use self::update_fleets_detected_acc::*;
use self::update_fleets_in_system::*;
use super::*;

impl<C> Metascape<C>
where
    C: ConnectionsManager,
{
    pub fn update_internal(&mut self) {
        unsafe {
            _TICK += 1;
            _TOTAL_TICK += 1;
        }

        let disconnected = connect_clients(self);
        disconnected.into_iter().for_each(|(connection, reason)| {
            handle_disconnect(self, connection, Some(reason));
        });

        let (disconnected, reasons) = handle_clients_inputs(self);
        disconnected
            .into_iter()
            .zip(reasons)
            .for_each(|(connection, reason)| {
                handle_disconnect(self, connection, Some(reason));
            });

        handle_faction_queue(self);

        handle_fleet_queue(self);

        // No more add/remove fleet from this point.

        // No more change to fleet's composition from this point.

        // TODO: AI

        apply_fleets_movement(self);

        update_fleets_in_system(self);

        update_fleets_detected_acc(self);

        // TODO: send factions infos the client request.

        let disconnected = send_detected_entities(self);
        disconnected.into_iter().for_each(|connection| {
            handle_disconnect(self, connection, None);
        })
    }
}
