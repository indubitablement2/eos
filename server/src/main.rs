#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(slice_split_at_unchecked)]
#![feature(iter_advance_by)]
#![feature(duration_constants)]
#![feature(derive_default_enum)]
#![feature(map_try_insert)]
#![feature(mixed_integer_ops)]
#![feature(map_entry_replace)]

use bevy_ecs::prelude::*;
use bevy_tasks::{ComputeTaskPool, TaskPool};
use colony::Colonies;
use common::factions::Factions;
use common::metascape_configs::MetascapeConfigs;
use common::systems::Systems;
use common::time::Time;
use common::{idx::SystemId, intersection::*};
use server_configs::ServerConfigs;
use data_manager::DataManager;
use clients_manager::*;
use fleets_manager::FleetsManager;
use std::{fs::File, io::prelude::*, thread::sleep, time::Instant};
use crate::terminal::Terminal;

mod colony;
mod connection_manager;
mod data_manager;
mod ecs_components;
mod ecs_events;
mod ecs_systems;
mod clients_manager;
mod fleets_manager;
mod terminal;
mod server_configs;
mod server;

// use common::metascape::Metascape;

#[macro_use]
extern crate log;



fn main() {
    tui_logger::init_logger(log::LevelFilter::Trace).expect("Could not init logger.");
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let mut metascape = Metascape::new().unwrap();
    let mut terminal = Terminal::new().expect("Could not create Terminal.");

    // Main loop.
    let mut loop_start = Instant::now();
    let mut stop_main = false;
    while !stop_main {
        // Time since last update.
        let delta = loop_start.elapsed();
        // Time alocated for this update.
        let update_duration =
            common::UPDATE_INTERVAL.saturating_sub(delta.saturating_sub(common::UPDATE_INTERVAL));
        // Update start time.
        loop_start = Instant::now();

        metascape.update();

        // Time used by the metacape update.
        let metascape_update_used_duration = loop_start.elapsed();

        // Update terminal.
        terminal.update(&mut stop_main, &mut metascape);

        // Time used in total.
        let total_update_used = loop_start.elapsed();
        // Time used by the terminal update.
        let terminal_update_used_duration = total_update_used - metascape_update_used_duration;

        // Update terminal performance metrics.
        terminal.update_performance_metrics(
            total_update_used.as_micros() as u64,
            metascape_update_used_duration.as_micros() as u64,
            terminal_update_used_duration.as_micros() as u64,
        );

        // Sleep for the remaining time.
        if let Some(remaining) = update_duration.checked_sub(total_update_used) {
            sleep(remaining);
        }
    }

    // Cleanup.
    terminal.clear();
}
