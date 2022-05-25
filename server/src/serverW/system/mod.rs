pub mod colony;

use self::colony::Colony;
use ahash::AHashSet;
use common::systems::Star;
use common::{idx::FleetId, systems::Planet};
use glam::Vec2;
use utils::*;

#[derive(Fields, Columns, Components)]
pub struct System {
    pub radius: f32,
    pub position: Vec2,
    pub star: Star,
    pub planets: Vec<Planet>,
    pub colonies: Vec<Colony>,
    pub fleets_in_system: AHashSet<FleetId>,
}
