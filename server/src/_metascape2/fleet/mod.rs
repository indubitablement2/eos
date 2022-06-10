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

    pub name: String,

    pub fleet_inner: FleetInner,

    /// If this fleet is within a system.
    pub in_system: Option<SystemId>,

    pub position: Vec2,
    pub velocity: Vec2,
    /// Where the fleet wish to move.
    pub wish_position: WishPosition,
    pub orbit: Option<common::orbit::Orbit>,

    /// How long this entity has been without velocity.
    pub idle_counter: idle_counter::IdleCounter,

    pub fleet_ai: FleetAi,
}

pub struct FleetBuilder {
    pub faction_id: FactionId,
    pub name: String,
    pub in_system: Option<SystemId>,
    pub position: Vec2,
    pub velocity: Vec2,
    pub wish_position: WishPosition,
    pub fleet_ai: FleetAi,
    pub fleet_composition: FleetComposition,
}
impl FleetBuilder {
    pub fn new(
        faction_id: FactionId,
        name: String,
        position: Vec2,
        fleet_ai: FleetAi,
        fleet_composition: FleetComposition,
    ) -> Self {
        Self {
            faction_id,
            name,
            in_system: None,
            position,
            velocity: Vec2::ZERO,
            wish_position: Default::default(),
            fleet_ai,
            fleet_composition,
        }
    }

    pub fn with_in_system(mut self, system_id: SystemId) -> Self {
        self.in_system = Some(system_id);
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

    pub fn build_ai(self) -> FleetId {
        let fleet_id = AI_FLEET_ID_DISPENSER.next();
        FLEET_QUEUE.push((fleet_id, self));
        fleet_id
    }

    pub fn build_client(self, client_id: ClientId) -> FleetId {
        let fleet_id = client_id.to_fleet_id();
        FLEET_QUEUE.push((fleet_id, self));
        fleet_id
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
        }
    }
}
