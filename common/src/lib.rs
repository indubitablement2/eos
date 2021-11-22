#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(slice_split_at_unchecked)]
#![feature(option_result_unwrap_unchecked)]

use crate::collision::*;
use crate::data_manager::DataManager;
use crate::ecs_events::add_event_handlers;
// use crate::{stage_last, stage_post_update, stage_pre_update, stage_update};
use crate::packets::ServerAddresses;
use crate::res_clients::ClientsRes;
use crate::res_factions::FactionsRes;
use crate::res_fleets::FleetsRes;
use crate::res_parameters::ParametersRes;
use crate::res_times::TimeRes;
// use crate::stage_first;
use crate::collision::Collider;
use bevy_ecs::prelude::*;
use bevy_tasks::TaskPool;
use generation::GenerationParameters;
use std::time::Duration;

#[macro_use]
extern crate log;

mod collision;
mod connection_manager;
mod ecs_components;
pub mod generation;
// mod metascape;
// pub mod metascape_new;
mod data_manager;
mod ecs_events;
mod ecs_systems;
mod fleet_ai;
pub mod packets;
mod res_clients;
mod res_factions;
mod res_fleets;
pub mod res_parameters;
mod res_times;

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
        ecs_systems::add_systems(&mut schedule);

        Ok(Self { world, schedule })
    }

    pub fn generate(local: bool, parameters: ParametersRes, generation_parameters: GenerationParameters) -> Self {
        todo!()
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
        self.world
            .get_resource::<IntersectionPipeline>()
            .unwrap()
            .get_colliders_copy()
    }
}
