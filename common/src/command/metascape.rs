use serde::{Serialize, Deserialize};
use crate::idx::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetascapeCommand {
    /// Set a fleet's wish position.
    FleetWishPosition {
        fleet_id: FleetId,
        target: na::Vector2<f32>,
        movement_multiplier: f32,
    },
    // TODO: Remove this
    ChatMessage {
        message: String,
    },
}

pub type TickCmd = (ClientId, MetascapeCommand);

pub struct MetascapeCommandList {
    command_list: Vec<Option<Vec<TickCmd>>>,
    pub last_tick: u64,
}
impl MetascapeCommandList {
    pub fn add_command_at_tick(&mut self, tick: u64, cmds: Vec<TickCmd>) {
        if tick <= self.last_tick {
            log::warn!("Tried to overwrite commands at tick {}. Ignoring...", tick);
            return;
        }

        self.command_list
            .resize(self.command_list.len().max((tick + 1) as usize), None);
        self.command_list[tick as usize] = Some(cmds);

        for new_tick in self.last_tick + 1..tick + 1 {
            if self
                .command_list
                .get(new_tick as usize)
                .and_then(|r| r.as_ref())
                .is_some()
            {
                self.last_tick = new_tick;
            }
        }

        log::debug!(
            "Received tick {}. New last tick is {}",
            tick,
            self.last_tick
        );
    }

    pub fn get_command_at_tick(&self, tick: u64) -> Option<&[TickCmd]> {
        self.command_list
            .get(tick as usize)
            .and_then(|r| r.as_ref().map(|r| r.as_slice()))
    }
}
