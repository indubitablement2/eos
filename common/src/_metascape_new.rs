use crate::collision::*;
use crate::data_manager::DataManager;
use crate::ecs_events::add_event_handlers;
use crate::{stage_last, stage_post_update, stage_pre_update, stage_update};
use crate::packets::ServerAddresses;
use crate::res_clients::ClientsRes;
use crate::res_factions::FactionsRes;
use crate::res_fleets::FleetsRes;
use crate::res_parameters::ParametersRes;
use crate::res_times::TimeRes;
use crate::stage_first;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use std::time::Duration;

pub struct Metascape {
    world: World,
    schedule: Schedule,
}
impl Metascape {
    /// How long between each Battlescape/Metascape tick.
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(50);

    pub fn new(local: bool, parameters: ParametersRes) -> std::io::Result<Self> {
        let mut world = World::new();
        add_event_handlers(&mut world);
        world.insert_resource(TaskPool::new());
        world.insert_resource(TimeRes::new());
        world.insert_resource(DataManager::new());
        world.insert_resource(parameters);
        world.insert_resource(IntersectionPipeline::new());
        world.insert_resource(ClientsRes::new(local));
        world.insert_resource(FactionsRes::new());
        world.insert_resource(FleetsRes::new());
        

        let mut schedule = Schedule::default();
        stage_first::add_systems(&mut schedule);
        stage_pre_update::add_systems(&mut schedule);
        stage_update::add_systems(&mut schedule);
        stage_post_update::add_systems(&mut schedule);
        stage_last::add_systems(&mut schedule);

        Ok(Self { world, schedule })
    }

    pub fn update(&mut self) {
        self.schedule.run_once(&mut self.world);
    }

    /// Get this server addressses.
    pub fn get_addresses(&self) -> ServerAddresses {
        self.world.get_resource::<ClientsRes>().unwrap().get_addresses()
    }

    /// Get a copy of every colliders separated by Membership. Useful for debug display.
    pub fn get_colliders(&self) -> Vec<Vec<Collider>> {
        self.world.get_resource::<IntersectionPipeline>().unwrap().get_colliders_copy()
    }
}
