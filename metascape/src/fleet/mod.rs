pub mod fleet_ai;
pub mod fleet_inner;
pub mod idle_counter;
pub mod wish_position;

use super::*;

use common::fleet::FleetComposition;
pub use fleet_ai::*;
pub use fleet_inner::*;
pub use idle_counter::*;
pub use wish_position::*;

#[derive(Soa, Serialize, Deserialize)]
pub struct Fleet {
    /// The faction this fleet is part of.
    pub faction: FactionId,
    pub masks: EnemyAlliedMasks,

    pub name: String,

    pub fleet_inner: FleetInner,

    /// If this fleet is within a system.
    pub in_system: Option<SystemId>,

    pub position: Vec2,
    pub velocity: Vec2,
    /// Where the fleet wish to move.
    pub wish_position: WishPosition,
    /// Orbit and the tick this was added.
    #[serde(skip)]
    pub orbit: Option<(common::orbit::Orbit, u32)>,

    /// How long this entity has been without velocity.
    #[serde(skip)]
    pub idle_counter: idle_counter::IdleCounter,

    pub fleet_ai: FleetAi,
}

/// ## Defaults:
/// - `faction`: FactionId::default().
/// - `name`: A random name will be generated.
/// - `fleet_ai`: Idle for npc, but always set to ClientControlled for client.
/// - `owner`: None, this is an npc fleet.
pub struct FleetBuilder {
    pub fleet_id: FleetId,
    pub faction: FactionId,
    /// Default to a generated name.
    pub name: Option<String>,
    pub position: Vec2,
    pub velocity: Vec2,
    /// Default to idle. Always set to ClientControlled for client.
    pub fleet_ai: Option<FleetAi>,
    pub fleet_composition: FleetComposition,
}
impl FleetBuilder {
    pub fn new_npc(position: Vec2, fleet_composition: FleetComposition) -> Self {
        Self {
            fleet_id: FLEET_ID_DISPENSER.next(),
            faction: Default::default(),
            name: None,
            position,
            velocity: Vec2::ZERO,
            fleet_ai: None,
            fleet_composition,
        }
    }

    pub fn new_client(
        client_id: ClientId,
        position: Vec2,
        fleet_composition: FleetComposition,
    ) -> Self {
        Self {
            fleet_id: client_id.to_fleet_id(),
            faction: Default::default(),
            name: None,
            position,
            velocity: Vec2::ZERO,
            fleet_ai: None,
            fleet_composition,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_fleet_ai(mut self, fleet_ai: FleetAi) -> Self {
        self.fleet_ai = Some(fleet_ai);
        self
    }
}
