use crate::{client_battlescape::ClientBattlescape, config::Config, inputs::*, ui::UiState};
use ahash::AHashMap;

pub struct State {
    config: Config,

    player_inputs: PlayerInputs,
    ui_state: UiState,

    bcs: AHashMap<u32, ClientBattlescape>,
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
        }
    }

    pub fn update(&mut self) {
        self.player_inputs.update(&self.config.input_map);

        let delta = macroquad::prelude::get_frame_time();
        for bc in self.bcs.values_mut() {
            bc.update(delta);
        }
    }

    pub fn draw(&mut self) {
        if let Some(bc) = self.bcs.values_mut().next() {
            bc.draw()
        }
    }

    pub fn draw_ui(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            // Draw battlescape ui.
            if let Some(bc) = self.bcs.values_mut().next() {
                bc.draw_ui(egui_ctx);
            }

            // TODO: Draw global ui.
        });
        self.ui_state.draw();
    }

    pub fn on_quit(self) {
        // TODO: If we have a battlescape, sent it to the server with checksum.
    }
}
