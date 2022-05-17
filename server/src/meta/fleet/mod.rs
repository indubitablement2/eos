pub mod idle_counter;
pub mod wish_position;

use common::idx::*;
use dioptre::Fields;
use glam::Vec2;
use soak::Columns;

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
    pub wish_position: wish_position::WishPosition,
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
impl Fleet {}
