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

use crossbeam::queue::SegQueue;

pub use ahash::{AHashMap, AHashSet};
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

/// Dispense unique and never recycled `FactionId`.
static FACTION_ID_DISPENSER: FactionIdDispenser = FactionIdDispenser::new();
static FACTION_QUEUE: SegQueue<(FactionId, FactionBuilder)> = SegQueue::new();

/// Dispense unique and never recycled `FleetId`.
static FLEET_ID_DISPENSER: NPCFleetIdDispenser = NPCFleetIdDispenser::new();
static FLEET_QUEUE: SegQueue<(FleetId, Fleet)> = SegQueue::new();

static mut _TICK: u32 = 0;
static mut _TOTAL_TICK: u64 = 0;
pub fn tick() -> u32 {
    unsafe { _TICK }
}

pub struct Metascape<C>
where
    C: ConnectionsManager,
{
    pub configs: Configs,
    pub rng: rand_xoshiro::Xoshiro256StarStar,

    pub connections_manager: C,
    pub connections: AHashMap<ClientId, ClientConnection<C::ConnectionType>>,
    pub authenticated: AHashMap<Auth, ClientId>,

    /// For fleets **outside** a system.
    pub fleets_out_detection_acc: AccelerationStructure<Circle, FleetId>,
    /// For fleets **inside** a system.
    pub fleets_in_detection_acc: AHashMap<SystemId, AccelerationStructure<Circle, FleetId>>,

    pub systems: Systems,
    /// System don't change. Never updated at runtime.
    pub systems_acceleration_structure: AccelerationStructure<Circle, SystemId>,

    pub clients: PackedMap<ClientSoa, ClientId>,
    pub fleets: PackedMap<FleetSoa, FleetId>,
    pub factions: PackedMap<FactionSoa, FactionId>,
}
impl<C> Metascape<C>
where
    C: ConnectionsManager,
{
    pub fn load(connections_manager: C, systems: Systems, configs: Configs, save: MetascapeSave) -> Self {
        // Load authenticated.
        let authenticated = AHashMap::from_iter(save.authenticated.into_iter());

        // Load clients.
        let clients = PackedMap::from_iter(save.clients.into_iter());

        // Load fleets.
        let fleets = PackedMap::from_iter(
            save.fleetsaves
                .into_iter()
                .map(|fleet_save| (fleet_save.fleet_id, fleet_save.to_fleet())),
        );

        // Load factions.
        let factions = PackedMap::from_iter(save.factions.into_iter());

        // Store statics variables.
        unsafe {
            FLEET_ID_DISPENSER.set(save.next_fleet_id);
            FACTION_ID_DISPENSER.set(save.next_faction_id);
            _TOTAL_TICK = save.total_tick;
        }

        Self {
            configs,
            connections_manager,
            systems_acceleration_structure: systems.create_acceleration_structure(),
            clients,
            authenticated,
            fleets,
            systems,
            factions,
            fleets_out_detection_acc: Default::default(),
            fleets_in_detection_acc: Default::default(),
            rng: rand_xoshiro::Xoshiro256StarStar::from_entropy(),
            connections: Default::default(),
        }
    }

    pub fn save(&mut self) -> MetascapeSave {
        log::error!("Save is not implemented yet. Returning default...");
        MetascapeSave::default()
    }

    pub fn update(&mut self) {
        self.update_internal();
    }
}

#[derive(Serialize, Deserialize)]
pub struct MetascapeSave {
    pub total_tick: u64,
    pub next_fleet_id: FleetId,
    pub next_faction_id: FactionId,
    pub fleetsaves: Vec<FleetSave>,
    pub factions: Vec<(FactionId, Faction)>,
    pub clients: Vec<(ClientId, Client)>,
    pub authenticated: Vec<(Auth, ClientId)>,
}
impl Default for MetascapeSave {
    fn default() -> Self {
        Self {
            total_tick: Default::default(),
            next_fleet_id: FleetId(1),
            next_faction_id: FactionId(0),
            fleetsaves: Default::default(),
            factions: Default::default(),
            clients: Default::default(),
            authenticated: Default::default(),
        }
    }
}
