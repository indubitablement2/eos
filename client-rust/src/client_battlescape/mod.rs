mod runner;
pub mod snapshop;

use self::runner::RunnerHandle;
use self::snapshop::BattlescapeSnapshot;
use crate::client::ClientConfig;
use crate::time_manager::*;
use battlescape::commands::Replay;
use battlescape::*;
use common::ClientId;
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
    pub fn new(base: &Node2D, replay: Replay, client_config: &ClientConfig) -> Self {
        // TODO: Take latest jump point.
        let bc = Battlescape::new(replay.initial_state);

        Self {
            catching_up: true,
            time_manager: TimeManager::new(client_config.battlescape_time_manager_config),
            snapshot: BattlescapeSnapshot::new(ClientId(0), base, &bc),
            runner_handle: RunnerHandle::new(bc),
            replay,
        }
    }

    pub fn update(&mut self, delta: f32) {
        let mut can_advance = None;
        let mut reset_time = false;
        if let Some((bc, events)) = self.runner_handle.update() {
            can_advance = Some(bc.tick);

            if self.catching_up {
                self.catching_up = (self.replay.cmds.len() as u64) - bc.tick > 20;
            } else {
                self.catching_up = (self.replay.cmds.len() as u64) - bc.tick > 40;
            }

            // Take snapshot for rendering.
            if !self.catching_up {
                reset_time = self.snapshot.take_snapshot(bc, events);
            }

            // log::debug!(
            //     "bc: {}, snapshot_render: {}, snapshot_max: {}, target: {}, max: {}, cmds: {}, t: {:.4}",
            //     bc.tick,
            //     self.snapshot.render_tick,
            //     self.snapshot.max_render_tick,
            //     self.time_manager.tick,
            //     self.time_manager.max_tick,
            //     self.replay.cmds.len(),
            //     self.time_manager.tick as f64 + self.time_manager.tick_frac as f64
            // );
        }

        if reset_time {
            self.time_manager.reset();
        }
        self.time_manager
            .maybe_max_tick(self.snapshot.max_tick().unwrap_or_default());
        self.time_manager.update(delta);
        // log::debug!("{}", self.time_manager.buffer_time_remaining());
        // log::debug!("t: {:.4}", self.time_manager.time_dilation);

        if let Some(cmds) = can_advance.and_then(|next_tick| self.replay.get_cmds(next_tick)) {
            // Apply jump point.
            if let Some((bytes, _)) = &cmds.jump_point {
                match Battlescape::load(bytes) {
                    Ok(new_bc) => {
                        self.runner_handle.bc = Some((Box::new(new_bc), Default::default()));
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

    pub fn draw(&mut self, base: &Node2D) {
        if self.catching_up || !self.snapshot.can_draw(self.time_manager.tick) {
            // TODO: Display catching up message.
        } else {
            self.snapshot.draw_lerp(
                self.time_manager.tick,
                self.time_manager.interpolation_weight(),
                base,
            );
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.snapshot.set_visible(visible);
    }
}
