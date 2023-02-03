pub mod render;
mod runner;

use self::render::BattlescapeRender;
use self::runner::RunnerHandle;
use super::*;
use crate::battlescape::{command::*, Battlescape, DT, DT_MS};
use crate::client_config::ClientConfig;
use crate::metascape::BattlescapeId;
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

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct ClientBattlescape {
    runner_handle: RunnerHandle,
    render: BattlescapeRender,
    client_type: ClientType,
    inputs: PlayerInputs,
    can_cheat: bool,
    wish_cmds: Commands,
    last_cmds_send: f32,
    /// Flag telling if we are very far behind.
    /// This will disable rendering and inputs to speed up simulation.
    catching_up: bool,
    time_manager: TimeManager<{ DT_MS }>,
    replay: Replay,
    #[base]
    base: Base<Node2D>,
}
impl ClientBattlescape {
    pub fn new(
        client_node: Gd<Node>,
        replay: Replay,
        client_config: &ClientConfig,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Gd<Self> {
        // TODO: Take latest jump point.
        let bc = Battlescape::new(replay.initial_state.clone());

        let (config, can_cheat) = match client_type {
            ClientType::Local => (TimeManagerConfig::local(), false),
            ClientType::LocalCheat => (TimeManagerConfig::local(), true),
            ClientType::Replay => (TimeManagerConfig::very_smooth(), false),
            ClientType::Client => (client_config.battlescape_time_manager_config, false),
        };

        Gd::with_base(|mut base: Base<Node2D>| {
            base.hide();

            Self {
            render: BattlescapeRender::new(client_id, client_node, &bc),
            runner_handle: RunnerHandle::new(bc),
            replay,
            wish_cmds: Default::default(),
            can_cheat,
            catching_up: true,
            time_manager: TimeManager::new(config),
            client_type,
            inputs: Default::default(),
            last_cmds_send: 0.0,
            base
        }})
    }

    pub fn battlescape_id(&self) -> BattlescapeId {
        self.replay.battlescape_id
    }
}
#[godot_api]
impl ClientBattlescape {
    #[func]
    fn can_cheat(&self) -> bool {
        self.can_cheat
    }

    #[func]
    fn sv_add_ship(&mut self, fleet_idx: u32, ship_idx: u32) {
        if self.can_cheat {
            self.wish_cmds.push(SvAddShip {
                fleet_id: FleetId(fleet_idx as u64),
                ship_idx,
                prefered_spawn_point: fleet_idx,
            });
        }
    }
}
#[godot_api]
impl GodotExt for ClientBattlescape {
    fn process(&mut self, delta: f64) {
        let delta = delta as f32;

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

        let hidden = !self.base.is_visible();

        if hidden || self.catching_up {
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

            // Add inputs cmd
            if !hidden && self.client_type != ClientType::Replay {
                cmds.push(SetClientInput {
                    client_id: self.render.client_id,
                    inputs: self.inputs.to_client_inputs(&self.base),
                });
            }

            if self.client_type == ClientType::Local || self.client_type == ClientType::LocalCheat {
                let tick = self.replay.next_needed_tick();
                self.replay.add_tick(tick, cmds);
            } else if !cmds.is_empty() && self.client_type != ClientType::Replay {
                // TODO: Send cmds to server
            }
        }
    }
}