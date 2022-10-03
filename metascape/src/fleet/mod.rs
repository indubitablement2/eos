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
    /// If a client own this fleet.
    pub client_owner: Option<ClientId>,

    pub name: String,

    pub fleet_inner: FleetInner,

    /// If this fleet is within a system.
    pub in_system: Option<SystemId>,

    pub position: na::Vector2<f32>,
    pub velocity: na::Vector2<f32>,
    /// Where the fleet wish to move.
    pub wish_position: WishPosition,
    /// If this fleet has an orbit.
    #[serde(skip)]
    pub orbit: Option<Orbit>,

    /// How long this entity has been without velocity.
    #[serde(skip)]
    pub idle_counter: IdleCounter,

    pub fleet_ai: FleetAi,
}

/// ## Defaults:
/// - `faction`: FactionId::default().
/// - `name`: A random name will be generated.
/// - `fleet_ai`: Idle for npc, but always set to ClientControlled for client.
pub struct FleetBuilder {
    pub fleet_id: FleetId,
    pub faction: FactionId,
    pub client_owner: Option<ClientId>,
    /// Default to a generated name.
    pub name: Option<String>,
    pub position: na::Vector2<f32>,
    pub velocity: na::Vector2<f32>,
    /// Default to idle. Always set to ClientControlled for client.
    pub fleet_ai: Option<FleetAi>,
    pub fleet_composition: FleetComposition,
}
impl FleetBuilder {
    /// If this is for a client: fleet id should be from client id.
    pub fn new(
        fleet_id: FleetId,
        position: na::Vector2<f32>,
        fleet_composition: FleetComposition,
    ) -> Self {
        Self {
            fleet_id,
            faction: Default::default(),
            name: None,
            position,
            velocity: na::zero(),
            fleet_ai: None,
            fleet_composition,
            client_owner: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_velocity(mut self, velocity: na::Vector2<f32>) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_fleet_ai(mut self, fleet_ai: FleetAi) -> Self {
        self.fleet_ai = Some(fleet_ai);
        self
    }

    pub fn with_client_owner(mut self, client_owner: ClientId) -> Self {
        self.client_owner = Some(client_owner);
        self
    }
}
