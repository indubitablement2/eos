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

// use common::metascape::Metascape;

#[macro_use]
extern crate log;

/// An acceleration structure that contain the systems bounds.
/// It is never updated at runtime.
pub struct SystemsAccelerationStructure(pub AccelerationStructure<SystemId, NoFilter>);

/// Acceleration structure with the `detected` colliders.
///
/// Filter are faction bitflags to which the entity is enemy or all 1s if the entity is neutral.
pub struct DetectedIntersectionPipeline(pub IntersectionPipeline<Entity, u32>);

pub struct Metascape {
    world: World,
    schedule: Schedule,
}
impl Metascape {
    fn new() -> std::io::Result<Self> {
        let mut world = World::new();
        ecs_events::add_event_res(&mut world);
        world.insert_resource(ComputeTaskPool(TaskPool::new()));
        world.insert_resource(Time::default());
        world.insert_resource(DataManager::new());
        world.insert_resource(DetectedIntersectionPipeline(IntersectionPipeline::new()));
        world.insert_resource(FleetsManager::default());

        // TODO: Load ServerConfigs from file.
        let server_configs = ServerConfigs::default();
        world.insert_resource(ClientsManager::new(&server_configs.clients_manager_configs)?);
        world.insert_resource(server_configs.connection_handler_configs);

        // TODO: Load MetascapeConfigs from file or use default.
        let metascape_configs = MetascapeConfigs::default();
        world.insert_resource(metascape_configs);


        // Load systems.
        let mut file = File::open("systems.bin").expect("Could not open systems.bin");
        let mut buffer = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buffer).unwrap();
        let mut systems =
            bincode::deserialize::<Systems>(&buffer).expect("Could not deserialize systems.bin");
        systems.update_all();

        // Load factions.
        let mut file = File::open("factions.yaml").expect("Could not open factions.bin");
        let mut buffer = String::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_string(&mut buffer).unwrap();
        let mut factions = serde_yaml::from_str::<Factions>(buffer.as_str())
            .expect("Could not deserialize factions.yaml");
        factions.update_all();

        // Load colonies.
        // TODO: This should be loaded from file.
        world.insert_resource(Colonies::default());

        // Add systems and systems_acceleration_structure resource.
        world.insert_resource(SystemsAccelerationStructure(
            systems.create_acceleration_structure(),
        ));
        world.insert_resource(systems);
        world.insert_resource(factions);

        // Create schedule.
        let mut schedule = Schedule::default();
        ecs_systems::add_systems(&mut schedule);

        Ok(Self { world, schedule })
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
