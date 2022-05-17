pub mod idle_counter;
pub mod wish_position;

use common::idx::*;
use dioptre::Fields;
use glam::Vec2;
use soak::Columns;

use self::wish_position::WishPosition;

#[derive(Fields, Columns)]
pub struct Fleet {
    /// Used internaly to mark invalid fleet and which tick this fleet was created.
    pub generation: u32,

    pub fleet_id: FleetId,
    pub faction_id: FactionId,

    /// If this fleet is within a system.
    pub in_system: Option<SystemId>,

    pub position: Vec2,
    pub velocity: Vec2,
    /// Where the fleet wish to move.
    pub wish_position: WishPosition,
    pub orbit: Option<common::orbit::Orbit>,

    /// How much space this fleet takes.
    pub radius: f32,

    /// Extra radius this fleet will get detected.
    pub detected_radius: f32,
    /// Radius this fleet will detect things.
    pub detector_radius: f32,
    /// Fleet detected by this fleet.
    pub fleet_detected: Vec<u32>,

    /// How long this entity has been without velocity.
    pub idle_counter: idle_counter::IdleCounter,
}

pub struct FleetBuilder {
        pub generation: u32,
        pub fleet_id: FleetId,
        pub faction_id: FactionId,
        pub in_system: Option<SystemId>,
        pub position: Vec2,
        pub velocity: Vec2,
        pub wish_position: WishPosition,
}
impl FleetBuilder {
    pub fn new(tick: u32, fleet_id: FleetId, faction_id: FactionId, position: Vec2) -> Self {
        Self {
            generation: tick,
            fleet_id,
            faction_id,
            in_system: None,
            position,
            velocity: Vec2::ZERO,
            wish_position: Default::default(),
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

    pub fn build(self) -> Fleet {
        Fleet {
            generation: self.generation,
            fleet_id: self.fleet_id,
            faction_id: self.faction_id,
            in_system: self.in_system,
            position: self.position,
            velocity: self.velocity,
            wish_position: self.wish_position,
            orbit: None,
            radius: 1.0, // TODO: Compute this.
            detected_radius: 10.0, // TODO: Compute this.
            detector_radius: 10.0, // TODO: Compute this.
            fleet_detected: Default::default(),
            idle_counter: Default::default(),
        }
    }
}
