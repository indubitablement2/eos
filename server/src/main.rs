#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(slice_split_at_unchecked)]
#![feature(iter_advance_by)]

use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use common::packets::ServerAddresses;
use data_manager::DataManager;
use generation::GenerationParameters;
use intersection::FleetIntersectionPipeline;
use res_clients::ClientsRes;
use res_factions::FactionsRes;
use res_fleets::FleetsRes;
use res_parameters::ParametersRes;
use res_system::SystemsRes;
use res_times::TimeRes;
use std::{thread::sleep, time::Instant};

use crate::terminal::Terminal;

mod connection_manager;
mod data_manager;
mod ecs_components;
mod ecs_events;
mod ecs_systems;
mod generation;
mod intersection;
mod res_clients;
mod res_factions;
mod res_fleets;
mod res_parameters;
mod res_system;
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
    fn new(parameters: ParametersRes, generation_parameters: GenerationParameters) -> std::io::Result<Self> {
        let mut world = World::new();
        ecs_events::add_event_res(&mut world);
        world.insert_resource(TaskPool::new());
        world.insert_resource(TimeRes::new());
        world.insert_resource(DataManager::new());
        let (system_res, system_intersection_pipeline) = SystemsRes::generate(&generation_parameters, &parameters);
        world.insert_resource(system_res);
        world.insert_resource(system_intersection_pipeline);
        world.insert_resource(parameters);
        world.insert_resource(FleetIntersectionPipeline::new());
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
                println!("{:?}", err);
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

fn startup() -> std::io::Result<Metascape> {
    // Ask if we should use default values.
    println!("Do you want to use default Metascape values? [_/n]");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    // Init Metascape.
    if buffer.split_whitespace().next().unwrap_or_default() == "n" {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "TODO"));
    } else {
        let generation_parameters = GenerationParameters::default();
        return Ok(Metascape::new(ParametersRes::default(), generation_parameters)?);
    }
}
