use crate::{idx::*, orbit::Orbit};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeInput {
    /// Toggle firing selected weapon group.
    pub fire_toggle: bool,
    /// The angle of the capital ship wish direction.
    pub wish_dir: f32,
    /// The angle of the capital ship's selected weapons wish direction.
    pub aim_dir: f32,
    /// The absolute force of the capital ship wish direction.
    pub wish_dir_force: f32,
}
impl Default for BattlescapeInput {
    fn default() -> Self {
        Self {
            fire_toggle: false,
            wish_dir: 0.0,
            aim_dir: 0.0,
            wish_dir_force: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeCommand {
    pub tick: u16,
    pub clients_inputs: Vec<BattlescapeInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeCommands {
    pub commands: Vec<BattlescapeCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesState {
    pub tick: u32,
    pub client_entity_position: Vec2,
    /// Entity's id and position compressed and relative to client's position.
    ///
    /// TODO: Compressed to 16 + 32 bits
    pub relative_entities_position: Vec<(u16, Vec2)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetInfo {
    pub fleet_id: FleetId,
    /// The ships composing the fleet.
    pub composition: Vec<ShipBaseId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityInfoType {
    Fleet(FleetInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub info_type: EntityInfoType,
    pub name: String,
    /// If this entity follow an orbit, its state will not be sent.
    pub orbit: Option<Orbit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesInfo {
    /// This is useful with orbit.
    /// Any state before this tick can be discarded and apply the orbit instead.
    /// Any state after this tick will remove the orbit.
    pub tick: u32,
    /// The client's (fleet) info, if it has changed.
    pub client_info: Option<EntityInfo>,
    pub infos: Vec<(u16, EntityInfo)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesRemove {
    pub tick: u32,
    /// Free these entities. Their idx will be reused in the future.
    pub to_remove: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectedReasonEnum {
    /// Server received an invalid packet.
    InvalidPacket,
    /// Someone else connected on the same account.
    ConnectionFromOther,
    /// The server has encountered a fatal error through no fault of the client.
    ServerError,
}
impl Display for DisconnectedReasonEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisconnectedReasonEnum::InvalidPacket => write!(f, "Server received an invalid packet."),
            DisconnectedReasonEnum::ConnectionFromOther => write!(f, "Someone else connected on the same account."),
            DisconnectedReasonEnum::ServerError => write!(f, "The server has encountered a fatal error through no fault of the client."),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Packet {
    /// Could not deserialize/serialize packet.
    #[default]
    Invalid,

    /// Server send this for every commands that are not acknowledged by the client.
    BattlescapeCommands(BattlescapeCommands),
    /// Server send some entities's position.
    EntitiesState(EntitiesState),
    /// Server send some entities's infos.
    EntitiesInfo(EntitiesInfo),
    /// Server send entities that will not be updated anymore and should be removed.
    EntitiesRemove(EntitiesRemove),
    /// Server send the reason why it disconnected the client.
    DisconnectedReason(DisconnectedReasonEnum),

    Message {
        /// Invalid ClientId means the origin is the server.
        origin: ClientId,
        content: String,
    },

    /// Client send this when he wants his fleet to move to a position.
    MetascapeWishPos { wish_pos: Vec2, movement_multiplier: f32 },
    /// Client send his battlescape inputs and last acknowledged commands.
    BattlescapeInput {
        wish_input: BattlescapeInput,
        /// The last Battlescape command the client acknowledge to have received.
        /// All commands before are implicitly acknowledged and will not be resent be the server.
        last_acknowledge_command: u16,
    },
}
impl Packet {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize::<Self>(buffer).unwrap_or_default()
    }
}
