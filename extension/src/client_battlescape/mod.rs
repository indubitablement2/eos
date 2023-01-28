pub mod render;
mod runner;

use self::render::BattlescapeRender;
use self::runner::RunnerHandle;
use super::*;
use crate::battlescape::bc_client::ClientInputs;
use crate::battlescape::{command::*, Battlescape, DT, DT_MS};
use crate::client_config::ClientConfig;
use crate::player_inputs::PlayerInputs;
use crate::time_manager::*;
use godot::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientType {
    /// Wish cmds are applied directly.
    Local,
    /// Local with the right to cheat.
    LocalCheat,
    /// Already has all the cmds.
    Replay,
    /// Send wish cmds to the server. Receive cmds from the server.
    Client,
}

pub struct ClientBattlescape {
    runner_handle: RunnerHandle,
    render: BattlescapeRender,
    client_type: ClientType,
    can_cheat: bool,
    wish_cmds: Commands,
    last_cmds_send: f32,
    /// Flag telling if we are very far behind.
    /// This will disable rendering and inputs to speed up simulation.
    catching_up: bool,
    time_manager: TimeManager<{ DT_MS }>,
    replay: Replay,
    hidden: bool,
}
impl ClientBattlescape {
    pub fn new(
        client_node: Gd<Node>,
        replay: Replay,
        client_config: &ClientConfig,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Self {
        // TODO: Take latest jump point.
        let bc = Battlescape::new(replay.initial_state.clone());

        let (config, can_cheat) = match client_type {
            ClientType::Local => (TimeManagerConfig::local(), false),
            ClientType::LocalCheat => (TimeManagerConfig::local(), true),
            ClientType::Replay => (TimeManagerConfig::very_smooth(), false),
            ClientType::Client => (client_config.battlescape_time_manager_config, false),
        };

        Self {
            render: BattlescapeRender::new(client_id, client_node, &bc),
            runner_handle: RunnerHandle::new(bc),
            replay,
            hidden: true,
            wish_cmds: Default::default(),
            can_cheat,
            catching_up: true,
            time_manager: TimeManager::new(config),
            client_type,
            last_cmds_send: 0.0,
        }
    }

    /// Return wish cmds that should be sent to the server.
    pub fn update(&mut self, delta: f32, inputs: Option<&PlayerInputs>) -> Option<Commands> {
        let mut can_advance = None;
        if let Some((bc, snapshot)) = self.runner_handle.update() {
            can_advance = Some(bc.tick + 1);

            let behind = self.replay.next_needed_tick() - bc.tick;
            if self.client_type == ClientType::Replay {
                self.catching_up = false;
                if behind > 6 {
                    can_advance = None;
                }
            } else if self.catching_up {
                self.catching_up = behind > 20;
            } else {
                self.catching_up = behind > 40;
            }

            // Take snapshot for rendering.
            if let Some(snapshot) = snapshot {
                if self.render.take_snapshot(bc, snapshot) {
                    self.time_manager.reset();
                }
            }
        }

        self.time_manager
            .maybe_max_tick(self.render.max_tick().unwrap_or_default());
        self.time_manager.update(delta);

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

        if self.hidden || self.catching_up {
            self.wish_cmds.clear();
            self.last_cmds_send = 0.0;
        } else {
            self.render.update(delta);
            self.render.draw_lerp(
                self.time_manager.tick,
                self.time_manager.interpolation_weight(),
            );
        }

        self.last_cmds_send += delta;
        if self.last_cmds_send > DT {
            self.last_cmds_send = 0.0;

            let mut cmds = std::mem::take(&mut self.wish_cmds);

            if let Some(inputs) = inputs {
                cmds.push(SetClientInput {
                    client_id: self.render.client_id,
                    inputs: inputs.to_client_inputs(),
                });
            }

            if self.client_type == ClientType::Local || self.client_type == ClientType::LocalCheat {
                let tick = self.replay.next_needed_tick();
                self.replay.add_tick(tick, cmds);
                None
            } else if cmds.is_empty() || self.client_type == ClientType::Replay {
                None
            } else {
                Some(cmds)
            }
        } else {
            None
        }
    }

    pub fn hide(&mut self, hide: bool) {
        self.hidden = hide;
        self.render.hide(hide);
    }
}
