use super::*;

impl Simulation {
    pub fn apply_commands(&mut self, cmds: &[Command], render_events: &mut RenderEvents) {
        for cmd in cmds {
            match cmd {
                Command::ShipMoveOrder(cmd) => self.ship_move_order(cmd),
            }
        }
    }

    fn ship_move_order(&mut self, cmd: &ShipMoveOrder) {
        for ship_id in cmd.ship_id.iter() {
            if let Some(ship) = self.ships.get_mut(ship_id) {
                ship.wish_pos = Some(cmd.position);
            }
        }
    }
}
