mod runner;
pub mod snapshop;

use self::runner::RunnerHandle;
use self::snapshop::BattlescapeSnapshot;
use crate::time_manager::*;
use battlescape::commands::Replay;
use battlescape::*;
use gdnative::prelude::*;

pub struct ClientBattlescape {
    /// Flag telling if we are very far behind.
    ///
    /// This will disable rendering and inputs to speed up simulation.
    pub catching_up: bool,
    pub replay: Replay,
    pub time_manager: TimeManager<{ Battlescape::TICK_DURATION_MS }>,
    pub runner_handle: RunnerHandle,
    snapshot: (BattlescapeSnapshot, BattlescapeSnapshot),
}
impl ClientBattlescape {
    pub fn new(battlescape_time_manager_config: TimeManagerConfig, replay: Replay) -> Self {
        // TODO: take latest jump point.
        let bc = Battlescape::new(replay.initial_state);

        let time_manager = TimeManager::new(battlescape_time_manager_config);

        Self {
            catching_up: true,
            replay,
            time_manager,
            runner_handle: RunnerHandle::new(bc),
            snapshot: (Default::default(), Default::default()),
        }
    }

    /// Return true if we should quit.
    pub fn process(&mut self, delta: f32) -> bool {
        let mut can_advance = false;
        if let Some(bc) = self.runner_handle.update() {
            can_advance = true;

            self.time_manager.maybe_max_tick(bc.tick);

            self.catching_up = (self.replay.cmds.len() as u64) - bc.tick > 40;

            // Take snapshot for rendering.
            if !self.catching_up {
                std::mem::swap(&mut self.snapshot.0, &mut self.snapshot.1);
                self.snapshot.1.take_snapshot(bc);
            }
        }

        self.time_manager.update(delta);

        if can_advance {
            if let Some(cmds) = self.replay.cmds.get(self.time_manager.tick as usize) {
                if let Some((bytes, _)) = &cmds.jump_point {
                    match Battlescape::load(bytes) {
                        Ok(new_bc) => {
                            self.runner_handle.bc = Some(Box::new(new_bc));
                            log::debug!("Applied jump point.");
                        }
                        Err(err) => {
                            log::error!("{:?} while loading battlescape.", err);
                            return false;
                        }
                    }
                }

                self.runner_handle.step(cmds.cmds.to_owned());
            }
        }

        false
    }

    pub fn draw(&mut self, owner: &Node2D) {
        if self.catching_up {
            // TODO: Display catching up message.
        } else {
            BattlescapeSnapshot::draw_lerp(
                &self.snapshot.0,
                &self.snapshot.1,
                owner,
                self.time_manager.interpolation_weight(),
            );
        }
    }
}
