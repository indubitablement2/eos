#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(slice_split_at_unchecked)]
#![feature(iter_advance_by)]
#![feature(duration_constants)]
#![feature(map_try_insert)]
#![feature(mixed_integer_ops)]
#![feature(map_entry_replace)]
#![feature(duration_consts_float)]
#![feature(is_sorted)]
#![feature(macro_metavar_expr)]
#![feature(is_some_with)]
#![feature(hash_drain_filter)]

use server::Server;
use std::{thread::sleep, time::Instant};

mod connection_manager;
mod metascape;
mod server;
mod terminal;

fn main() {
    tui_logger::init_logger(log::LevelFilter::Trace).expect("Could not init logger.");
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let mut server = Server::new();

    let mut loop_start = Instant::now();
    loop {
        // Time since last update.
        let delta = loop_start.elapsed();
        // Time alocated for this update.
        let update_duration =
            common::TICK_DURATION.saturating_sub(delta.saturating_sub(common::TICK_DURATION));
        // Update start instant.
        loop_start = Instant::now();

        if server.update() {
            break;
        }

        // Sleep for the remaining time.
        if let Some(remaining) = update_duration.checked_sub(loop_start.elapsed()) {
            sleep(remaining);
        }
    }

    server.clear_terminal();
}
