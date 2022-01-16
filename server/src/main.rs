#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(slice_split_at_unchecked)]
#![feature(iter_advance_by)]
#![feature(duration_constants)]
#![feature(derive_default_enum)]
#![feature(map_try_insert)]

use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use common::parameters::Parameters;
use common::time::Time;
use common::world_data::WorldData;
use common::{idx::SystemId, intersection::*};
use data_manager::DataManager;
use res_clients::ClientsRes;
use res_fleets::FleetsRes;
use std::{fs::File, io::prelude::*, thread::sleep, time::Instant};

use crate::terminal::Terminal;

mod connection_manager;
mod data_manager;
mod ecs_components;
mod ecs_events;
mod ecs_systems;
mod res_clients;
mod res_fleets;
mod terminal;

// use common::metascape::Metascape;

#[macro_use]
extern crate log;

/// An acceleration structure that contain the systems bounds.
/// It is never updated at runtime.
pub struct SystemsAccelerationStructure(pub AccelerationStructure<SystemId>);

/// Acceleration structure with the `detected` colliders.
pub struct DetectedIntersectionPipeline(pub IntersectionPipeline<Entity>);

pub struct Metascape {
    world: World,
    schedule: Schedule,
}
impl Metascape {
    fn new(local: bool, parameters: Parameters) -> std::io::Result<Self> {
        let mut world = World::new();
        ecs_events::add_event_res(&mut world);
        world.insert_resource(TaskPool::new());
        world.insert_resource(Time::default());
        world.insert_resource(DataManager::new());
        world.insert_resource(parameters);
        world.insert_resource(DetectedIntersectionPipeline(IntersectionPipeline::new()));
        world.insert_resource(ClientsRes::new(local)?);
        world.insert_resource(FleetsRes::new());

        let mut schedule = Schedule::default();
        ecs_systems::add_systems(&mut schedule);

        Ok(Self { world, schedule })
    }

    fn load(&mut self) {
        // Load world data.
        let mut file = File::open("world_data.bin").expect("Could not find world_data.bin");
        let mut buffer = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buffer).unwrap();
        let wd = bincode::deserialize::<WorldData>(&buffer).expect("Could not deserialize world_data.bin");

        // Create an acceleration structure for systems.
        let mut acc = AccelerationStructure::new();
        acc.extend(
            wd.systems
                .iter()
                .map(|(system_id, system)| (Collider::new(system.bound, system.position), system_id.to_owned())),
        );
        acc.update();

        // Add systems and systems_acceleration_structure resource.
        self.world.insert_resource(wd);
        self.world.insert_resource(SystemsAccelerationStructure(acc));
    }

    fn update(&mut self) {
        self.schedule.run_once(&mut self.world);
        unsafe {
            self.world
                .get_resource_unchecked_mut::<Time>()
                .unwrap_unchecked()
                .increment();
        }
        self.world.clear_trackers();
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

    println!("Loading...");
    metascape.load();

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
    let mut buffer = String::new();

    // Ask if we should create local server.
    println!("Do you want to create a server over localhost? [_/n]");
    std::io::stdin().read_line(&mut buffer)?;
    let local = buffer.split_whitespace().next().unwrap_or_default() != "n";
    println!("Local server: {}", local);

    // Ask if we should use default values.
    println!("Do you want to use default Metascape values? [_/n]");
    buffer.clear();
    std::io::stdin().read_line(&mut buffer)?;
    let default_values = buffer.split_whitespace().next().unwrap_or_default() != "n";
    println!("Default values: {}", default_values);

    // Init Metascape.
    if default_values {
        return Ok(Metascape::new(local, Parameters::default())?);
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TODO: Non default values",
        ));
    }
}
