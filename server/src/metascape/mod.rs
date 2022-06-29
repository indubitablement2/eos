pub mod client;
pub mod colony;
mod connection;
pub mod faction;
pub mod fleet;
pub mod id_dispenser;
pub mod server_configs;
mod update;

use crate::connection_manager::ConnectionsManager;
use common::net::connection::Connection;
use crossbeam::queue::SegQueue;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

// TODO: Separate into a prelude file.
pub use ahash::{AHashMap, AHashSet};
pub use client::*;
pub use common::idx::*;
pub use common::net::packets::*;
pub use common::orbit::*;
pub use common::reputation::*;
pub use common::system::*;
pub use common::*;
pub use connection::*;
pub use faction::*;
pub use fleet::*;
pub use glam::Vec2;
pub use id_dispenser::*;
pub use rand::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use server_configs::*;
pub use soa_derive::*;
pub use utils::{acc::*, *};

/// Dispense unique and never recycled `FactionId`.
static FACTION_ID_DISPENSER: FactionIdDispenser = FactionIdDispenser::new();
static FACTION_QUEUE: SegQueue<(FactionId, FactionBuilder)> = SegQueue::new();

/// Dispense unique and never recycled `FleetId`.
static FLEET_ID_DISPENSER: FleetIdDispenser = FleetIdDispenser::new();
static FLEET_QUEUE: SegQueue<(FleetId, Fleet)> = SegQueue::new();

static mut _TICK: u32 = 0;
static mut _TOTAL_TICK: u64 = 0;
pub fn tick() -> u32 {
    unsafe { _TICK }
}

pub struct Metascape {
    pub server_configs: ServerConfigs,
    pub rt: Arc<tokio::runtime::Runtime>,
    pub rng: rand_xoshiro::Xoshiro256StarStar,

    pub connections_manager: ConnectionsManager,
    pub pendings_connection: VecDeque<Connection>,

    /// For fleets **outside** a system.
    pub fleets_out_detection_acc: AccelerationStructure<Circle, FleetId>,
    /// For fleets **inside** a system.
    pub fleets_in_detection_acc: AHashMap<SystemId, AccelerationStructure<Circle, FleetId>>,

    pub systems: Systems,
    /// System don't change. Never updated at runtime.
    pub systems_acceleration_structure: AccelerationStructure<Circle, SystemId>,

    pub clients: PackedMap<ClientSoa, ClientId>,
    pub connections: PackedMap<ConnectionSoa, ClientId>,
    pub fleets: PackedMap<FleetSoa, FleetId>,
    pub factions: PackedMap<FactionSoa, FactionId>,
}
impl Metascape {
    pub fn load() -> Self {
        // TODO: Load data.
        common::data::Data::default().init();

        // TODO: Load server configs.
        let server_configs = ServerConfigs::default();

        // Load systems.
        let mut file = File::open("systems.bin").expect("could not open systems.bin");
        let mut buffer = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buffer).unwrap();
        let systems =
            bincode::deserialize::<Systems>(&buffer).expect("could not deserialize systems.bin");

        // Create async runtime.
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        );

        let connections_manager =
            ConnectionsManager::new(server_configs.connection_configs.local, &rt).unwrap();

        // TODO: Load MetascapeSave.
        let save = MetascapeSave::default();

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
            server_configs,
            rt,
            connections_manager,
            pendings_connection: Default::default(),
            systems_acceleration_structure: systems.create_acceleration_structure(),
            clients,
            fleets,
            systems,
            factions,
            fleets_out_detection_acc: Default::default(),
            fleets_in_detection_acc: Default::default(),
            rng: rand_xoshiro::Xoshiro256StarStar::from_entropy(),
            connections: PackedMap::with_capacity(256),
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
        }
    }
}
