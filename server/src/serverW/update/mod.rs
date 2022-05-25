mod apply_fleets_movement;
mod connect_clients;
mod update_fleets_acc;
mod update_fleets_in_system;
mod handle_clients_inputs;

use self::apply_fleets_movement::*;
use self::connect_clients::*;
use self::update_fleets_acc::*;
use self::update_fleets_in_system::*;
use self::handle_clients_inputs::*;
use super::*;

impl Server {
    pub fn update_internal(&mut self) {
        self.time.increment();
        
        connect_clients(self);

        handle_clients_inputs(self);
        
        // TODO: AI

        apply_fleets_movement(self);

        update_fleets_detection_acceleration_structure(self);
        update_fleets_in_system(self);
    }
}
