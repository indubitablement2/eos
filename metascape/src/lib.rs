#![feature(hash_drain_filter)]
#![feature(drain_filter)]
#![feature(is_some_with)]

pub mod client;
pub mod colony;
pub mod configs;
pub mod faction;
pub mod fleet;
pub mod id_dispenser;
mod update;

extern crate nalgebra as na;

pub use acc::*;
pub use ahash::{AHashMap, AHashSet};
use bincode::Options;
pub use bit_vec::BitVec;
pub use client::*;
pub use common::idx::*;
pub use common::net::*; // TODO: remove
pub use common::orbit::*;
pub use common::rand_vector::*;
pub use common::reputation::*;
pub use common::system::*;
pub use common::*;
pub use configs::*;
pub use faction::*;
pub use fleet::*;
pub use id_dispenser::*;
pub use rand::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use soa_derive::Soa;
pub use utils::packed_map::*;

pub type Fleets = PackedMap<FleetSoa, FleetId>;
pub type Factions = AHashMap<FactionId, Faction>;
pub type Clients = AHashMap<ClientId, Client>;

#[derive(Serialize, Deserialize)]
pub struct Metascape {
    pub configs: Configs,
    pub rng: rand_xoshiro::Xoshiro256StarStar,

    /// Number of step since the metascape was first created.
    pub tick: u64,

    /// For fleets **outside** a system.
    #[serde(skip)] // Will be created as needed.
    pub fleets_out_detection_acc: Sap<FleetId, CircleBoundingShape>,
    /// For fleets **inside** a system.
    #[serde(skip)] // Will be created as needed.
    pub fleets_in_detection_acc: AHashMap<SystemId, Sap<FleetId, CircleBoundingShape>>,

    pub systems: Systems,
    /// System don't change. Never updated at runtime.
    #[serde(skip)] // Computed from `systems`.
    pub systems_acceleration_structure: Sap<SystemId, CircleBoundingShape>,

    pub factions: Factions,
    pub factions_reputation: FactionReputations,

    pub clients: Clients,

    pub fleet_id_dispenser: FleetIdDispenser,
    pub fleets: Fleets,
}
impl Metascape {
    pub fn new(configs: Configs, systems: Systems, factions: Factions) -> Self {
        // TODO: Random generation.

        Self {
            configs,
            systems,
            factions,
            ..Default::default()
        }
    }

    pub fn load(buffer: &[u8]) -> Option<Self> {
        let mut s = bincode::options().deserialize::<Self>(buffer).ok()?;
        s.init();
        Some(s)
    }

    pub fn save(&self) -> Vec<u8> {
        // Afaik this can not fail.
        bincode::options().serialize(&self).unwrap()
    }

    fn init(&mut self) {
        self.systems_acceleration_structure = self.systems.create_acceleration_structure();
    }

    pub fn step(&mut self) {
        self.update_internal();
    }
}

impl Default for Metascape {
    fn default() -> Self {
        Self {
            configs: Default::default(),
            rng: rand_xoshiro::Xoshiro256StarStar::seed_from_u64(1337),
            fleets_out_detection_acc: Default::default(),
            fleets_in_detection_acc: Default::default(),
            systems: Default::default(),
            systems_acceleration_structure: Default::default(),
            factions: Default::default(),
            factions_reputation: Default::default(),
            clients: Default::default(),
            fleets: Default::default(),
            tick: Default::default(),
            fleet_id_dispenser: Default::default(),
        }
    }
}
