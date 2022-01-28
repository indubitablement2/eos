use bevy_ecs::prelude::*;
use bevy_tasks::{ComputeTaskPool, TaskPool};
use common::factions::Factions;
use common::metascape_configs::MetascapeConfigs;
use common::systems::Systems;
use common::time::Time;
use common::{idx::SystemId, intersection::*};
use crate::colony::Colonies;
use crate::server_configs::ServerConfigs;
use crate::data_manager::DataManager;
use crate::clients_manager::*;
use crate::fleets_manager::FleetsManager;
use std::{fs::File, io::prelude::*};

/// An acceleration structure that contain the systems bounds.
/// It is never updated at runtime.
pub struct SystemsAccelerationStructure(pub AccelerationStructure<SystemId, NoFilter>);

/// Acceleration structure with the `detected` colliders.
///
/// Filter are faction bitflags to which the entity is enemy or all 1s if the entity is neutral.
pub struct DetectedIntersectionPipeline(pub IntersectionPipeline<Entity, u32>);

pub struct Server {
    world: World,
    schedule: Schedule,
}
impl Server {
    fn new() -> std::io::Result<Self> {
        let mut world = World::new();
        crate::ecs_events::add_event_res(&mut world);
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
        crate::ecs_systems::add_systems(&mut schedule);

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