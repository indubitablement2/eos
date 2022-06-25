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

#[derive(Soa)]
pub struct Fleet {
    pub faction_id: FactionId,
    /// If a client own this fleet.
    pub owner: Option<ClientId>,

    pub name: String,

    pub fleet_inner: FleetInner,

    /// If this fleet is within a system.
    pub in_system: Option<SystemId>,

    pub position: Vec2,
    pub velocity: Vec2,
    /// Where the fleet wish to move.
    pub wish_position: WishPosition,
    /// Orbit and the tick this was added.
    pub orbit: Option<(common::orbit::Orbit, u32)>,

    /// How long this entity has been without velocity.
    pub idle_counter: idle_counter::IdleCounter,

    pub fleet_ai: FleetAi,
}

/// ## Defaults:
/// - `faction_id`: A new faction will be created.
/// - `name`: A random name will be generated.
/// - `fleet_ai`: Idle for npc, but always set to ClientControlled for client.
/// - `owner`: None, this is an npc fleet.
pub struct FleetBuilder {
    /// Default to creating a new faction.
    pub faction_id: Option<FactionId>,
    /// Default to a generated name.
    pub name: Option<String>,
    pub position: Vec2,
    pub velocity: Vec2,
    pub wish_position: WishPosition,
    /// Default to idle. Always set to ClientControlled for client.
    pub fleet_ai: Option<FleetAi>,
    pub fleet_composition: FleetComposition,
    pub owner: Option<ClientId>,
}
impl FleetBuilder {
    pub fn new(position: Vec2, fleet_composition: FleetComposition) -> Self {
        Self {
            faction_id: None,
            name: None,
            position,
            velocity: Vec2::ZERO,
            wish_position: Default::default(),
            fleet_ai: None,
            fleet_composition,
            owner: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_faction(mut self, faction_id: FactionId) -> Self {
        self.faction_id = Some(faction_id);
        self
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_wish_position(mut self, wish_position: WishPosition) -> Self {
        self.wish_position = wish_position;
        self
    }

    pub fn with_fleet_ai(mut self, fleet_ai: FleetAi) -> Self {
        self.fleet_ai = Some(fleet_ai);
        self
    }

    pub fn with_owner(mut self, owner: ClientId) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Fleet will be created next update.
    pub fn build(self) -> FleetId {
        let fleet_id = FLEET_ID_DISPENSER.next();
        FLEET_QUEUE.push((fleet_id, self.to_fleet()));
        fleet_id
    }

    fn to_fleet(self) -> Fleet {
        Fleet {
            faction_id: self
                .faction_id
                .unwrap_or_else(|| FACTION_ID_DISPENSER.next()),
            name: self.name.unwrap_or_else(|| "todo!".to_string()), // TODO: Random name generation.
            fleet_inner: FleetInner::new(self.fleet_composition),
            in_system: None,
            position: self.position,
            velocity: self.velocity,
            wish_position: self.wish_position,
            orbit: None,
            idle_counter: Default::default(),
            fleet_ai: self.fleet_ai.unwrap_or_default(),
            owner: self.owner,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FleetSave {
    pub fleet_id: FleetId,
    pub faction_id: FactionId,
    pub name: String,
    pub position: Vec2,
    pub fleet_composition: FleetComposition,
    pub fleet_ai: FleetAi,
    pub owner: Option<ClientId>,
}
impl FleetSave {
    pub fn to_fleet(self) -> Fleet {
        Fleet {
            faction_id: self.faction_id,
            name: self.name,
            in_system: Default::default(),
            position: self.position,
            velocity: Default::default(),
            wish_position: Default::default(),
            orbit: Default::default(),
            idle_counter: Default::default(),
            fleet_ai: self.fleet_ai,
            fleet_inner: FleetInner::new(self.fleet_composition),
            owner: self.owner,
        }
    }
}
