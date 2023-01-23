pub mod render;
mod runner;

use self::render::BattlescapeRender;
use self::runner::RunnerHandle;
use super::*;
use crate::battlescape::{command::*, Battlescape, DT_MS};
use crate::client_config::ClientConfig;
use crate::time_manager::*;
use godot::prelude::*;

pub struct ClientBattlescape {
    /// Flag telling if we are very far behind.
    /// This will disable rendering and inputs to speed up simulation.
    catching_up: bool,
    time_manager: TimeManager<{ DT_MS }>,
    runner_handle: RunnerHandle,
    render: BattlescapeRender,
    replay: Replay,
    hide: bool,
}
impl ClientBattlescape {
    pub fn new(
        client_node: Gd<Node>,
        replay: Replay,
        client_config: &ClientConfig,
        client_id: ClientId,
    ) -> Self {
        // TODO: Take latest jump point.
        let bc = Battlescape::new(replay.initial_state.clone());

        Self {
            catching_up: true,
            time_manager: TimeManager::new(client_config.battlescape_time_manager_config),
            render: BattlescapeRender::new(client_id, client_node, &bc),
            runner_handle: RunnerHandle::new(bc),
            replay,
            hide: false,
        }
    }

    pub fn update(&mut self, delta: f32) {
        let mut can_advance = None;
        let mut reset_time = false;
        if let Some((bc, snapshot)) = self.runner_handle.update() {
            can_advance = Some(bc.tick + 1);

            if self.catching_up {
                self.catching_up = self.replay.next_needed_tick() - bc.tick > 20;
            } else {
                self.catching_up = self.replay.next_needed_tick() - bc.tick > 40;
            }

            // Take snapshot for rendering.
            if let Some(snapshot) = snapshot {
                reset_time = self.render.take_snapshot(bc, snapshot);
            }

            if bc.tick == 100 {
                // let checksum = crc32fast::hash(&bc.serialize());
                // panic!("{}", checksum);
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
            .maybe_max_tick(self.render.max_tick().unwrap_or_default());
        self.time_manager.update(delta);
        // log::debug!("{}", self.time_manager.buffer_time_remaining());
        // log::debug!("t: {:.4}", self.time_manager.time_dilation);

        if let Some(cmds) = can_advance.and_then(|next_tick| self.replay.get_cmds(next_tick)) {
            // TODO: Apply jump point. Do we need this to keep sim deteministic?
            // if let Some((bytes, _)) = &cmds.jump_point {
            //     match Battlescape::load(bytes) {
            //         Ok(new_bc) => {
            //             self.runner_handle.bc = Some((Box::new(new_bc), Default::default()));
            //             log::debug!("Applied jump point.");
            //         }
            //         Err(err) => {
            //             log::error!("{:?} while loading battlescape.", err);
            //         }
            //     }
            // }

            self.runner_handle.step(cmds.to_owned(), !self.catching_up);
        }

        if !self.hide && !self.catching_up {
            self.render.update(delta);
            self.render.draw_lerp(
                self.time_manager.tick,
                self.time_manager.interpolation_weight(),
            );
        }
    }

    pub fn hide(&mut self, visible: bool) {
        self.hide = visible;
        self.render.hide(visible);
    }
}
