mod runner;

use gdnative::api::*;
use gdnative::prelude::*;

use crate::time_manager::*;
use self::runner::RunnerHandle;
use battlescape::*;

pub struct ClientBattlescape {
    pub cmds: (),
    pub time_manager: TimeManager<{Battlescape::TICK_DURATION_MS}>,
    pub runner_handle: RunnerHandle,
}
impl ClientBattlescape {
    fn new(
        battlescape_time_manager_config: TimeManagerConfig,

    ) -> Self {

        let bc = Battlescape::new(Default::default());

        Self {
            cmds: (),
            time_manager: TimeManager::new(battlescape_time_manager_config),
            runner_handle: RunnerHandle::new(bc),
        }
    }

    // #[method]
    // unsafe fn _draw(&mut self, #[base] owner: &Node2D) {
    //     self.metascape_manager.draw(owner);
    // }

    pub fn process(&mut self, delta: f32) {
        self.time_manager.update(delta);

        let mut should_step = false;
        
        if let Some(bc) = self.runner_handle.update() {
            should_step = self.time_manager.tick > bc.tick;
        }

        if should_step {
            self.runner_handle.step(vec![])
        }
    }
}
