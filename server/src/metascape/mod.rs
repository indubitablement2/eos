mod battlescape_manager;
mod clients_manager;
mod colony;
mod data_manager;
mod ecs_components;
mod ecs_events;
mod ecs_systems;
mod fleets_manager;
mod interception_manager;
mod utils;

use bevy_ecs::prelude::*;
use bevy_tasks::{ComputeTaskPool, TaskPool};
use common::factions::Factions;
use common::metascape_configs::MetascapeConfigs;
use common::ships::Bases;
use common::systems::Systems;
use common::time::Time;
use common::{idx::SystemId, intersection::*};
use std::{fs::File, io::prelude::*};
use crate::server_configs::ServerConfigs;
use self::battlescape_manager::BattlescapeManager;
use self::clients_manager::ClientsManager;
use self::colony::Colonies;
use self::data_manager::DataManager;
use self::fleets_manager::FleetsManager;
use self::interception_manager::InterceptionManager;

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
    pub fn new() -> Self {
        let mut world = World::new();
        ecs_events::add_event_res(&mut world);
        world.insert_resource(ComputeTaskPool(TaskPool::new()));
        world.insert_resource(Time::default());
        world.insert_resource(DataManager::new());
        world.insert_resource(DetectedIntersectionPipeline(IntersectionPipeline::new()));
        world.insert_resource(FleetsManager::default());
        world.insert_resource(InterceptionManager::default());

        // TODO: Load ServerConfigs from file.
        let server_configs = ServerConfigs::default();
        world.insert_resource(
            ClientsManager::new(&server_configs.clients_manager_configs)
                .expect("Could not initialize ClientManager."),
        );
        world.insert_resource(server_configs.connection_handler_configs);

        // TODO: Load MetascapeConfigs from file or use default.
        let metascape_configs = MetascapeConfigs::default();
        world.insert_resource(metascape_configs);

        // TODO: Load BattlescapeManager.
        world.insert_resource(BattlescapeManager::default());

        // TODO: Load bases.
        world.insert_resource(Bases::default());

        // Load systems.
        let mut file = File::open("systems.bin").expect("Could not open systems.bin");
        let mut buffer = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buffer).unwrap();
        let mut systems =
            bincode::deserialize::<Systems>(&buffer).expect("Could not deserialize systems.bin");
        systems.update_all();

        // Load factions.
        let mut file = File::open("factions.bin").expect("Could not open factions.bin");
        let mut buffer = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buffer).unwrap();
        let mut factions = bincode::deserialize::<Factions>(&buffer)
            .expect("Could not deserialize factions.yaml");
        world.insert_resource(factions);

        // Load colonies.
        // TODO: This should be loaded from file.
        world.insert_resource(Colonies::default());

        // Add systems and systems_acceleration_structure resource.
        world.insert_resource(SystemsAccelerationStructure(
            systems.create_acceleration_structure(),
        ));
        world.insert_resource(systems);

        // Create schedule.
        let mut schedule = Schedule::default();
        self::ecs_systems::add_systems(&mut schedule);

        Self { world, schedule }
    }

    /// Run the metascape's schedule once.
    pub fn update(&mut self) {
        self.schedule.run_once(&mut self.world);
        self.world.clear_trackers();
    }
}
