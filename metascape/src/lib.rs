#![feature(hash_drain_filter)]
#![feature(drain_filter)]
#![feature(is_some_with)]

pub mod client;
mod client_connection;
pub mod colony;
pub mod configs;
pub mod faction;
pub mod fleet;
pub mod id_dispenser;
mod update;

use bincode::Options;
use crossbeam::queue::SegQueue;

pub use ahash::{AHashMap, AHashSet};
pub use bit_vec::BitVec;
pub use client::*;
pub use client_connection::*;
pub use common::idx::*;
pub use common::net::*;
pub use common::orbit::*;
pub use common::reputation::*;
pub use common::system::*;
pub use common::*;
pub use configs::*;
pub use faction::*;
pub use fleet::*;
pub use glam::Vec2;
pub use id_dispenser::*;
pub use rand::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use soa_derive::*;
pub use utils::{acc::*, *};

/// Dispense unique and never recycled `FleetId`.
static FLEET_ID_DISPENSER: NPCFleetIdDispenser = NPCFleetIdDispenser::new();
static FLEET_QUEUE: SegQueue<(FleetId, Fleet)> = SegQueue::new();

pub type Tick = u32;
static mut _TICK: Tick = 0;
static mut _TOTAL_TICK: u64 = 0;
pub fn tick() -> u32 {
    unsafe { _TICK }
}

#[derive(Serialize, Deserialize)]
pub struct Metascape<C>
where
    C: ConnectionsManager,
{
    pub configs: Configs,
    pub rng: rand_xoshiro::Xoshiro256StarStar,

    #[serde(skip)]
    pub connections: AHashMap<ClientId, ClientConnection<C::ConnectionType>>,
    pub authenticated: AHashMap<Auth, ClientId>,

    /// For fleets **outside** a system.
    #[serde(skip)] // Will be created as needed.
    pub fleets_out_detection_acc: AccelerationStructure<Circle, FleetId>,
    /// For fleets **inside** a system.
    #[serde(skip)] // Will be created as needed.
    pub fleets_in_detection_acc: AHashMap<SystemId, AccelerationStructure<Circle, FleetId>>,

    pub systems: Systems,
    /// System don't change. Never updated at runtime.
    #[serde(skip)] // Computed from `systems`.
    pub systems_acceleration_structure: AccelerationStructure<Circle, SystemId>,

    pub factions: Factions,

    pub clients: PackedMap<ClientSoa, ClientId>,
    pub fleets: PackedMap<FleetSoa, FleetId>,
}
impl<C> Metascape<C>
where
    C: ConnectionsManager,
{
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

    pub fn save(&self) -> Option<Vec<u8>> {
        bincode::options().serialize(&self).ok()
    }

    fn init(&mut self) {
        self.systems_acceleration_structure = self.systems.create_acceleration_structure();
    }

    pub fn update(&mut self, connections_manager: &mut C) {
        self.update_internal(connections_manager);
    }
}

impl<C> Default for Metascape<C>
where
    C: ConnectionsManager,
{
    fn default() -> Self {
        Self {
            configs: Default::default(),
            rng: rand_xoshiro::Xoshiro256StarStar::from_entropy(),
            connections: Default::default(),
            authenticated: Default::default(),
            fleets_out_detection_acc: Default::default(),
            fleets_in_detection_acc: Default::default(),
            systems: Default::default(),
            systems_acceleration_structure: Default::default(),
            factions: Default::default(),
            clients: Default::default(),
            fleets: Default::default(),
        }
    }
}
