use crate::prelude::*;
use crate::{
    client_battlescape::ClientBattlescape, config::Config, inputs::*, rendering::Rendering,
    ui::UiState,
};
use ahash::AHashMap;
use battlescape::Battlescape;

pub struct State {
    rt: Runtime,

    config: Config,

    player_inputs: PlayerInputs,
    ui_state: UiState,

    rendering: Rendering,

    bcs: AHashMap<u32, ClientBattlescape>,

    // TODO: Remove.
    t: f32,
    u: u64,
}
impl State {
    pub fn init() -> Self {
        // TODO: Try to load from file.
        let config = Config::default();

        Self {
            player_inputs: Default::default(),
            ui_state: UiState::init(),
            bcs: AHashMap::from_iter(
                [(0, ClientBattlescape::new(Default::default(), &config))].into_iter(),
            ),
            config,
            rendering: Default::default(),
            rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            t: 0.0,
            u: 0,
        }
    }

    pub fn update(&mut self) {
        self.player_inputs.update(&self.config.input_map);

        let delta = macroquad::prelude::get_frame_time();
        self.t += delta;
        let inc = if self.t >= Battlescape::TICK_DURATION_SEC {
            self.t -= Battlescape::TICK_DURATION_SEC;
            self.u += 1;
            true
        } else {
            false
        };
        for bc in self.bcs.values_mut() {
            if inc {
                bc.replay.push_cmds(self.u - 1, Default::default());
            }
            bc.update(delta);
        }
    }

    pub fn draw(&mut self) {
        if let Some(bc) = self.bcs.values_mut().next() {
            bc.draw(&mut self.rendering)
        }

        self.rendering.draw(&self.rt);
    }

    pub fn draw_ui(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            // Draw battlescape ui.
            if let Some(bc) = self.bcs.values_mut().next() {
                bc.draw_ui(egui_ctx);
            }

            // TODO: Draw global ui.
        });
        egui_macroquad::draw();
        // self.ui_state.draw();
    }

    pub fn on_quit(self) {
        // TODO: If we have a battlescape, sent it to the server with checksum.
    }
}
