// Used in command
#![feature(hash_drain_filter)]

use env_logger::Env;
use eos_common::const_var::*;
use std::thread::sleep;
use std::time::{Duration, Instant};

mod command;
pub mod ecs;
pub mod ecs_component;
pub mod ecs_resoure;
pub mod ecs_system;
pub mod ecs_world;
pub mod global;
mod login;

#[macro_use]
extern crate log;

fn main() {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "trace")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env);

    info!("Starting server...");

    // Disconnect
    // let (to_disconnect_sender, to_disconnect_receiver) = crossbeam_channel::unbounded::<client::ClientData>();

    // GlobalList.
    let mut global_list_wrapper = global::GlobalListWrapper::new(0); // TODO: Fetch FleetId last counter.

    // SpaceGrid.
    let (mut space_grid, sector_senders) = ecs_world::SpaceGrid::new(&global_list_wrapper);

    // Connection.
    let mut polling_thread = eos_common::connection_manager::PollingThread::new(true);

    // Login.
    let login = login::LoginThread::new(polling_thread.connection_starter.clone());

    // ServerCommand.
    let mut server_command = command::ServerCommand::new();

    info!("Server ready");

    // If we should break main loop.
    let mut exit = false;
    // If we should accept new Connection.
    let mut accept_login = true;

    // * Main loop.
    let mut tick = 0u32;
    let mut average = Duration::ZERO;
    while !exit {
        let loop_start = Instant::now();
        tick += 1;

        // * Process server commands.
        server_command.process_command(&mut exit, &mut accept_login, &global_list_wrapper.global_list);

        // * Update each sector.
        space_grid.update();

        // * Get new Connection from login loop.
        if accept_login {
            login.process_login(&global_list_wrapper.global_list, &sector_senders);
        }

        // * Update global list.
        global_list_wrapper.update();

        // * Poll each connection.
        polling_thread.poll();

        // * Sleep for remainder of the loop.
        let remaining_time = MAIN_LOOP_DURATION - loop_start.elapsed();
        average += remaining_time;
        if tick % 100 == 0 {
            average /= 100;
            info!("Average time remaining: {:?}", &average);
            average = Duration::ZERO;
        }
        if remaining_time <= Duration::ZERO {
            error!("Server is lagging: {:?}", remaining_time);
        } else {
            sleep(remaining_time);
        }
    }

    info!("Main loop done");

    // disconnect_all(&connected_clients, &disconnected_clients, &to_disconnect);

    // TODO: Save.

    info!("Exit successful");
}
