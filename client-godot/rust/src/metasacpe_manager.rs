use crate::input_handler::PlayerInputs;
use crate::time_manager::*;
use crate::{godot_client_config::GodotClientConfig, metascape_runner::MetascapeRunnerHandle};
use common::*;
use gdnative::prelude::*;
use metascape::{Metascape, MetascapeCommandList};

#[derive(Default, Clone, Copy, Debug)]
enum ClientState {
    #[default]
    Server,
    Client,
}

pub struct MetascapeManager {
    pub take_save: bool,

    pub update_debug_info: bool,
    pub last_debug_info: String,

    player_inputs: PlayerInputs,

    client_state: ClientState,
    metascape_time_manager: TimeManager<METASCAPE_TICK_DURATION_MIL>,
    metascape_runner_handle: MetascapeRunnerHandle,
    metascape_command_list: MetascapeCommandList,

    godot_client_config: GodotClientConfig,
}
impl MetascapeManager {
    pub fn new(metascape: Metascape) -> Self {
        // TODO: Try to load from disk.
        let godot_client_config = GodotClientConfig::default();

        Self {
            last_debug_info: Default::default(),
            client_state: Default::default(),
            metascape_command_list: Default::default(),
            metascape_runner_handle: MetascapeRunnerHandle::new(metascape),
            take_save: false,
            update_debug_info: false,
            metascape_time_manager: TimeManager::new(
                godot_client_config
                    .server_metascape_time_manager_configs
                    .to_owned(),
            ),
            godot_client_config,
            player_inputs: Default::default(),
        }
    }

    pub fn unhandled_input(&mut self, event: TRef<InputEvent>) {
        self.player_inputs.handle_input(event);
    }

    pub fn process(&mut self, owner: &Node2D, delta: f32) {
        self.metascape_time_manager.update(delta);

        if let Some(metascape) = self.metascape_runner_handle.update() {
            // Take debug info.
            if self.update_debug_info {
                self.last_debug_info = take_debug_info(&metascape, self.client_state);
                self.update_debug_info = false;
            }

            // Take save.
            if self.take_save {
                let save = metascape.save();
                // TODO: Save to disk.
                self.take_save = false;
            }
        }
    }

    pub fn draw(&mut self, owner: &Node2D) {
        // if let ClientState::Connected(client_metascape) = &mut self.client_state {
        //     client_metascape.render(owner);
        // }
    }
}
impl Default for MetascapeManager {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

fn take_debug_info(metascape: &Metascape, client_state: ClientState) -> String {
    format!(
        "TIME:
    Tick: {}
NET:
    {:?}
    Num clients: {}
SYSTEM:
    Bound: {:.2}
    Num systems: {}
    Num planet: {}
METASCAPE:
    Num fleet: {}
    Num faction: {}
        ",
        metascape.tick,
        client_state,
        metascape.clients.len(),
        metascape.systems.bound,
        metascape.systems.systems.len(),
        metascape.systems.total_num_planet,
        metascape.fleets.len(),
        metascape.factions.len(),
    )
}
