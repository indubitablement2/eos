mod runner;

use battlescape::commands::Replay;
use gdnative::api::*;
use gdnative::prelude::*;
use crate::time_manager::*;
use self::runner::RunnerHandle;
use battlescape::*;

pub struct ClientBattlescape {
    /// Flag telling if we are very far behind.
    /// 
    /// This will disable rendering and inputs to speed up simulation.
    pub catching_up: bool,
    pub replay: Replay,
    pub time_manager: TimeManager<{Battlescape::TICK_DURATION_MS}>,
    pub runner_handle: RunnerHandle,
}
impl ClientBattlescape {
    fn new(
        battlescape_time_manager_config: TimeManagerConfig,
        replay: Replay,
    ) -> Self {
        // TODO: take latest jump point.
        let bc = Battlescape::new(replay.initial_state);
        
        let time_manager = TimeManager::new(battlescape_time_manager_config);

        Self {
            catching_up: true,
            replay,
            time_manager,
            runner_handle: RunnerHandle::new(bc),
        }
    }

    // #[method]
    // unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
    //     self.metascape_manager.draw(owner);
    // }

    /// Return true if this should quit.
    pub fn process(&mut self, delta: f32) -> bool {
        let mut behind = false;
        if let Some(bc) = self.runner_handle.update() {
            behind = self.time_manager.tick > bc.tick;
            
            self.catching_up = (self.replay.cmds.len() as u64) - bc.tick > 40;

            if self.replay.cmds.len() as u64 > bc.tick {
                self.time_manager.maybe_max_tick(bc.tick + 1);
            }

            // TODO: Take snapshot for rendering.
        }

        if self.time_manager.update(delta) || behind {
            let cmds = &self.replay.cmds[self.time_manager.tick as usize];

            if let Some((bytes, _)) = &cmds.jump_point {
                match Battlescape::load(bytes) {
                    Ok(new_bc) => {
                        self.runner_handle.bc = Some(Box::new(new_bc));
                        log::debug!("Applied jump point.");
                    }
                    Err(err) =>  {
                        log::error!("{:?} while loading battlescape.", err);
                        return false;
                    }
                }
            }

            self.runner_handle.step(cmds.cmds.to_owned());
        }

        false
    }
}
