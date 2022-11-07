mod runner;
pub mod snapshop;

use self::runner::RunnerHandle;
use self::snapshop::BattlescapeSnapshot;
use crate::client::ClientConfig;
use crate::time_manager::*;
use battlescape::*;
use battlescape::commands::Replay;
use gdnative::prelude::*;

pub struct ClientBattlescape {
    /// Flag telling if we are very far behind.
    ///
    /// This will disable rendering and inputs to speed up simulation.
    pub catching_up: bool,
    pub time_manager: TimeManager<{ Battlescape::TICK_DURATION_MS }>,
    pub runner_handle: RunnerHandle,
    pub snapshot: BattlescapeSnapshot,
    pub replay: Replay,
}
impl ClientBattlescape {
    pub fn new(replay: Replay, client_config: &ClientConfig) -> Self {
        // TODO: Take latest jump point.
        let bc = Battlescape::new(replay.initial_state);

        Self {
            catching_up: true,
            time_manager: TimeManager::new(client_config.battlescape_time_manager_config),
            runner_handle: RunnerHandle::new(bc),
            snapshot: Default::default(),
            replay,
        }
    }

    pub fn update(&mut self, owner: &Node2D, delta: f32) {
        let mut can_advance = None;
        if let Some(bc) = self.runner_handle.update() {
            can_advance = Some(bc.tick);

            let last_tick = bc.tick.saturating_sub(1);

            self.time_manager.maybe_max_tick(last_tick);

            self.catching_up = (self.replay.cmds.len() as u64) - last_tick > 40;

            // Take snapshot for rendering.
            if !self.catching_up {
                self.snapshot.update(bc);
            }

            // log::debug!(
            //     "last: {}, bc: {}, target: {}, max: {}, cmds: {}, t: {:.4}",
            //     last_tick,
            //     bc.tick,
            //     self.time_manager.tick,
            //     self.time_manager.max_tick,
            //     data.replay.cmds.len(),
            //     self.time_manager.tick as f32 + self.time_manager.tick_frac
            // );
        }

        self.time_manager.update(delta);
        log::debug!("t: {:.4}", self.time_manager.time_dilation);

        if let Some(next_tick) = can_advance {
            if let Some(cmds) = self.replay.cmds.get(next_tick as usize) {
                // Apply jump point.
                if let Some((bytes, _)) = &cmds.jump_point {
                    match Battlescape::load(bytes) {
                        Ok(new_bc) => {
                            self.runner_handle.bc = Some(Box::new(new_bc));
                            log::debug!("Applied jump point.");
                        }
                        Err(err) => {
                            log::error!("{:?} while loading battlescape.", err);
                        }
                    }
                }

                self.runner_handle.step(cmds.cmds.to_owned());
            }
        }
    }

    pub fn draw(&mut self, base: &Node2D) {
        if self.catching_up {
            // TODO: Display catching up message.
        } else {
            self.snapshot.draw_lerp(self.time_manager.interpolation_weight(), base);
        }
    }
}
