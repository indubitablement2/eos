#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(slice_split_at_unchecked)]
#![feature(option_result_unwrap_unchecked)]

use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use collision::{Collider, IntersectionPipeline};
use common::packets::ServerAddresses;
use data_manager::DataManager;
use res_clients::ClientsRes;
use res_factions::FactionsRes;
use res_fleets::FleetsRes;
use res_parameters::ParametersRes;
use res_times::TimeRes;
use std::{thread::sleep, time::Instant};

use crate::terminal::Terminal;

mod collision;
mod connection_manager;
mod data_manager;
mod ecs_components;
mod ecs_events;
mod ecs_systems;
mod fleet_ai;
mod generation;
mod res_clients;
mod res_factions;
mod res_fleets;
mod res_parameters;
mod res_times;
mod terminal;

// use common::metascape::Metascape;

#[macro_use]
extern crate log;

pub struct Metascape {
    world: World,
    schedule: Schedule,
}
impl Metascape {
    fn new(parameters: ParametersRes) -> std::io::Result<Self> {
        let mut world = World::new();
        ecs_events::add_event_res(&mut world);
        world.insert_resource(TaskPool::new());
        world.insert_resource(TimeRes::new());
        world.insert_resource(DataManager::new());
        world.insert_resource(parameters);
        world.insert_resource(IntersectionPipeline::new());
        world.insert_resource(ClientsRes::new()?);
        world.insert_resource(FactionsRes::new());
        world.insert_resource(FleetsRes::new());

        let mut schedule = Schedule::default();
        ecs_systems::add_systems(&mut schedule);

        Ok(Self { world, schedule })
    }

    // pub fn generate(local: bool, parameters: ParametersRes, generation_parameters: GenerationParameters) -> Self {
    //     todo!()
    // }

    fn update(&mut self) {
        self.schedule.run_once(&mut self.world);
    }

    /// Get this server addressses.
    fn get_addresses(&self) -> ServerAddresses {
        self.world.get_resource::<ClientsRes>().unwrap().get_addresses()
    }

    /// Get a copy of every colliders separated by Membership. Useful for debug display.
    fn get_colliders(&self) -> Vec<Vec<Collider>> {
        self.world
            .get_resource::<IntersectionPipeline>()
            .unwrap()
            .get_colliders_copy()
    }
}

fn main() {
    tui_logger::init_logger(log::LevelFilter::Trace).expect("Could not init logger.");
    tui_logger::set_default_level(log::LevelFilter::Trace);

    let mut metascape;
    loop {
        match startup() {
            Ok(new_metascape) => {
                metascape = new_metascape;
                break;
            }
            Err(err) => {
                warn!("{:?}", err);
            }
        }
    }

    let mut terminal = Terminal::new().expect("Could not create Terminal.");

    // Main loop.
    let mut loop_start = Instant::now();
    let mut stop_main = false;
    while !stop_main {
        // Time since last update.
        let delta = loop_start.elapsed();
        // Time alocated for this update.
        let update_duration = common::UPDATE_INTERVAL.saturating_sub(delta.saturating_sub(common::UPDATE_INTERVAL));
        // Update start time.
        loop_start = Instant::now();
        trace!("Last main loop duration: {} ms.", delta.as_millis());
        trace!("Next main loop expected duration: {} ms.", update_duration.as_millis());

        metascape.update();
        terminal.update(&mut stop_main, &mut metascape);

        // Sleep for the remaining time.
        if let Some(remaining) = update_duration.checked_sub(loop_start.elapsed()) {
            sleep(remaining);
        }
    }

    // Cleanup.
    terminal.clear();
}

fn startup() -> std::io::Result<Metascape> {
    // Ask if we should use default values.
    println!("Do you want to use default Metascape values? [_/n]");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    // Init Metascape.
    if buffer.split_whitespace().next().unwrap_or_default() == "n" {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "TODO"));
    } else {
        return Ok(Metascape::new(ParametersRes::default())?);
    }
}
