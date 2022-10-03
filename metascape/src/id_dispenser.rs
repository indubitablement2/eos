use super::*;

/// Dispense unique and never recycled `FleetId`.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FleetIdDispenser {
    next_fleet_id: FleetId,
}
impl FleetIdDispenser {
    /// Get the next fleet id and increment the inner counter.
    pub fn next(&mut self) -> FleetId {
        let id = self.next_fleet_id;
        self.next_fleet_id.0 += 1;
        id
    }
}
