mod client_battlescape;
mod config;
mod inputs;
mod logger;
mod prelude;
mod state;
mod time_manager;
mod ui;
mod utils;

use macroquad::prelude::*;
use state::State;

extern crate nalgebra as na;

#[macroquad::main("BasicShapes")]
async fn main() {
    logger::Logger::init();

    set_pc_assets_folder("assets");
    prevent_quit();

    let mut state = State::init();

    loop {
        // TODO: No need to clear bg.
        clear_background(BLACK);

        state.update();
        state.draw();
        state.draw_ui();

        if is_quit_requested() {
            state.on_quit();
            break;
        }

        next_frame().await
    }

    logger::write_logs_to_file();
}
