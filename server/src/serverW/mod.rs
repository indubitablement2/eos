pub mod faction;
pub mod fleet;
pub mod server_configs;
pub mod system;
mod update;
pub mod client;

use std::sync::Arc;
use std::collections::VecDeque;
use common::net::connection::Connection;
use crate::connection_manager::ConnectionsManager;

pub use self::client::*;
pub use self::fleet::*;
pub use self::server_configs::*;
pub use self::system::*;
pub use common::idx::*;
pub use common::time::*;
pub use common::net::packets::*;
pub use faction::*;
pub use glam::Vec2;
pub use utils::{acc::*, *};

pub struct Server {
    pub server_configs: ServerConfigs,
    pub rt: Arc<tokio::runtime::Runtime>,

    pub connections_manager: ConnectionsManager,
    pub pendings_connection: VecDeque<Connection>,

    pub time: Time,
    /// Use the fleet's Current system id as filter or u32::MAX no fleet not in a system.
    pub fleets_detection_acceleration_structure: AccelerationStructure<FleetId, u32>,
    /// System don't move. Never updated at runtime.
    pub systems_acceleration_structure: AccelerationStructure<SystemId, ()>,

    pub clients: PackedMap<Soa<Client>, Client, ClientId>,
    pub fleets: PackedMap<Soa<Fleet>, Fleet, FleetId>,
    pub systems: PackedMap<Soa<System>, System, SystemId>,
    pub factions: PackedMap<Soa<Faction>, Faction, FactionId>,
}
impl Server {
    pub fn update(&mut self) {
        self.update_internal();
    }
}
