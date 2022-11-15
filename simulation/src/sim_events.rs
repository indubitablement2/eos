use super::*;

#[derive(Debug, Clone, Default)]
pub struct SimulationEvents {
    /// Ship moved to another system.
    pub ship_moved: Vec<ShipId>,
}
