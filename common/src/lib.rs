#![feature(test)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(slice_split_at_unchecked)]
#![feature(option_result_unwrap_unchecked)]

use collision::Collider;
use generation::GenerationParameters;
use metascape::{FactionId, Metascape};
use packets::ServerAddresses;
use std::time::Duration;

#[macro_use]
extern crate log;

mod collision;
mod connection_manager;
mod ecs_components;
pub mod generation;
mod metascape;
mod metascape_new;
pub mod packets;
mod res_clients;
mod res_factions;
mod res_fleets;
pub mod res_parameters;
mod res_times;
mod stage_start;

// pub const SIZE_SMALL_FLEET: f32 = 0.1;
// pub const SIZE_GAUGING_NORMAL_PLANET: f32 = 1.0;
// pub const SIZE_NORMAL_STAR: f32 = 4.0;

pub struct MetascapeParameters {
    /// The maximum distance to the center.
    pub bound: f32,
    /// Multiply fleet velocity every tick.
    pub movement_friction: f32,
}

pub struct MetascapeWrapper {
    metascape: Metascape,
}
impl MetascapeWrapper {
    /// How long between each Battlescape/Metascape tick.
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(50);

    /// Create a new Metascape with default parameters.
    pub fn new(local: bool, parameters: MetascapeParameters) -> tokio::io::Result<Self> {
        Ok(Self {
            metascape: Metascape::new(local, parameters)?,
        })
    }

    /// Generate the Metascape.
    pub fn generate(&mut self, generation_parameters: &mut GenerationParameters) {
        generation_parameters.generate(&mut self.metascape);
    }

    pub fn update(&mut self) {
        self.metascape.update();
    }

    /// Get a copy of every colliders. Useful for debug display.
    pub fn get_colliders(&self) -> Vec<Collider> {
        self.metascape.intersection_pipeline.get_colliders_copy()
    }

    pub fn get_addresses(&self) -> ServerAddresses {
        self.metascape.connection_manager.get_addresses()
    }

    pub fn get_faction_relation(&self) -> Vec<(FactionId, Vec<(FactionId, f32)>)> {
        self.metascape
            .faction
            .iter()
            .map(|(faction_id, faction)| (*faction_id, faction.get_pretty_relation()))
            .collect()
    }
}
