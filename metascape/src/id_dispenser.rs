use super::*;

/// Dispense unique and never recycled `FleetId`.
#[derive(Debug, Serialize, Deserialize)]
pub struct FleetIdDispenser {
    next_fleet_id: FleetId,
}
impl FleetIdDispenser {
    /// Get the next npc fleet id and increment the inner counter.
    pub fn next(&mut self) -> FleetId {
        let id = self.next_fleet_id;
        self.next_fleet_id.0 += 1;
        id
    }
}
impl Default for FleetIdDispenser {
    fn default() -> Self {
        // First u32::MAX are reserved for client.
        Self {
            next_fleet_id: FleetId(u32::MAX as u64 + 1),
        }
    }
}
