pub mod client;
pub mod colony;
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

pub use self::client::*;
pub use self::fleet::*;
pub use self::id_dispenser::*;
pub use self::server_configs::*;
pub use common::idx::*;
pub use common::net::packets::*;
pub use common::systems::*;
pub use common::time::*;
pub use faction::*;
pub use glam::Vec2;
pub use serde::{Deserialize, Serialize};
pub use utils::{acc::*, *};

static FACTION_ID_DISPENSER: FactionIdDispenser = FactionIdDispenser::new();
static FACTION_QUEUE: SegQueue<(FactionId, FactionBuilder)> = SegQueue::new();

static AI_FLEET_ID_DISPENSER: AiFleetIdDispenser = AiFleetIdDispenser::new();
static FLEET_QUEUE: SegQueue<(FleetId, FleetBuilder)> = SegQueue::new();

pub struct Metascape {
    pub server_configs: ServerConfigs,
    pub rt: Arc<tokio::runtime::Runtime>,

    pub connections_manager: ConnectionsManager,
    pub pendings_connection: VecDeque<Connection>,

    pub time: Time,
    /// Use the fleet's Current system id as filter or u32::MAX no fleet not in a system.
    pub fleets_detection_acceleration_structure: AccelerationStructure<FleetId, u32>,
    /// System don't move. Never updated at runtime.
    pub systems_acceleration_structure: AccelerationStructure<SystemId, ()>,
    pub bound: f32,

    pub clients: PackedMap<Soa<Client>, Client, ClientId>,
    pub fleets: PackedMap<Soa<Fleet>, Fleet, FleetId>,
    pub systems: PackedMap<Soa<System>, System, SystemId>,
    pub factions: PackedMap<Soa<Faction>, Faction, FactionId>,
}
impl Metascape {
    pub fn load() -> Self {
        // TODO: Load server configs.
        let server_configs = ServerConfigs::default();

        // Load systems.
        let mut file = File::open("systems.bin").expect("could not open systems.bin");
        let mut buffer = Vec::with_capacity(file.metadata().unwrap().len() as usize);
        file.read_to_end(&mut buffer).unwrap();
        let mut systems_data = bincode::deserialize::<common::systems::Systems>(&buffer)
            .expect("could not deserialize systems.bin");
        systems_data.update_all();
        let mut systems_acceleration_structure = AccelerationStructure::new();
        let mut systems = PackedMap::with_capacity(systems_data.systems.len());
        for (system_id, system) in systems_data.systems {
            systems_acceleration_structure
                .push(Collider::new(system.position, system.radius, ()), system_id);
            systems.insert(system_id, system);
        }
        systems_acceleration_structure.update();

        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        );

        let connections_manager =
            ConnectionsManager::new(server_configs.connection_configs.local, &rt).unwrap();

        // TODO: Load MetascapeSave.
        let metascape_save = MetascapeSave::default();

        // Load fleets.
        let mut fleets = PackedMap::with_capacity(metascape_save.fleetsaves.len());
        for fleet_save in metascape_save.fleetsaves {
            fleets.insert(
                fleet_save.fleet_id,
                Fleet {
                    faction_id: fleet_save.faction_id,
                    name: fleet_save.name,
                    in_system: Default::default(),
                    position: fleet_save.position,
                    velocity: Default::default(),
                    wish_position: Default::default(),
                    orbit: Default::default(),
                    composition: fleet_save.composition,
                    acceleration: Default::default(),
                    radius: Default::default(),
                    detected_radius: Default::default(),
                    detector_radius: Default::default(),
                    idle_counter: Default::default(),
                    fleet_ai: fleet_save.fleet_ai,
                    last_change: 1, // This will cause a recompute
                },
            );
        }

        // Load factions.
        let mut factions = PackedMap::with_capacity(metascape_save.factions.len());
        for (faction_id, faction) in metascape_save.factions {
            factions.insert(faction_id, faction);
        }

        // Store statics variables.
        unsafe {
            AI_FLEET_ID_DISPENSER.set(metascape_save.next_ai_fleet_id);
            FACTION_ID_DISPENSER.set(metascape_save.next_faction_id);
        }

        Self {
            server_configs,
            rt,
            connections_manager,
            pendings_connection: Default::default(),
            time: Time {
                tick: 0,
                total_tick: metascape_save.total_tick,
            },
            fleets_detection_acceleration_structure: AccelerationStructure::new(),
            systems_acceleration_structure,
            clients: PackedMap::with_capacity(256),
            fleets,
            systems,
            factions,
            bound: systems_data.bound,
        }
    }

    pub fn save(&mut self) -> MetascapeSave {
        let mut fleetsaves = Vec::with_capacity(self.fleets.len());
        let (faction_id, name, position, composition, fleet_ai) = query_ptr!(
            self.fleets,
            Fleet::faction_id,
            Fleet::name,
            Fleet::position,
            Fleet::composition,
            Fleet::fleet_ai
        );
        for (i, fleet_id) in self.fleets.id_vec().iter().enumerate() {
            let (faction_id, name, position, composition, fleet_ai) = unsafe {
                (
                    &*faction_id.add(i),
                    &*name.add(i),
                    &*position.add(i),
                    &*composition.add(i),
                    &*fleet_ai.add(i),
                )
            };

            fleetsaves.push(FleetSave {
                fleet_id: fleet_id.to_owned(),
                faction_id: faction_id.to_owned(),
                name: name.to_owned(),
                position: position.to_owned(),
                composition: composition.to_owned(),
                fleet_ai: fleet_ai.to_owned(),
            });
        }

        let mut factions = Vec::with_capacity(self.factions.len());
        let (name, reputations, fallback_reputation, clients, fleets, colonies) = query_ptr!(
            self.factions,
            Faction::name,
            Faction::reputations,
            Faction::fallback_reputation,
            Faction::clients,
            Faction::fleets,
            Faction::colonies,
        );
        for (i, faction_id) in self.factions.id_vec().iter().enumerate() {
            let (name, reputations, fallback_reputation, clients, fleets, colonies) = unsafe {
                (
                    &*name.add(i),
                    &*reputations.add(i),
                    &*fallback_reputation.add(i),
                    &*clients.add(i),
                    &*fleets.add(i),
                    &*colonies.add(i),
                )
            };

            factions.push((
                faction_id.to_owned(),
                Faction {
                    name: name.to_owned(),
                    reputations: reputations.to_owned(),
                    fallback_reputation: fallback_reputation.to_owned(),
                    clients: clients.to_owned(),
                    fleets: fleets.to_owned(),
                    colonies: colonies.to_owned(),
                },
            ));
        }

        MetascapeSave {
            total_tick: self.time.total_tick,
            next_ai_fleet_id: unsafe { AI_FLEET_ID_DISPENSER.current() },
            next_faction_id: unsafe { FACTION_ID_DISPENSER.current() },
            fleetsaves,
            factions,
        }
    }

    pub fn update(&mut self) {
        self.update_internal();
    }
}

#[derive(Serialize, Deserialize)]
pub struct MetascapeSave {
    pub total_tick: u64,
    pub next_ai_fleet_id: FleetId,
    pub next_faction_id: FactionId,
    pub fleetsaves: Vec<FleetSave>,
    pub factions: Vec<(FactionId, Faction)>,
}
impl Default for MetascapeSave {
    fn default() -> Self {
        Self {
            total_tick: Default::default(),
            next_ai_fleet_id: FleetId(u32::MAX as u64 + 1),
            next_faction_id: FactionId(0),
            fleetsaves: Default::default(),
            factions: Default::default(),
        }
    }
}