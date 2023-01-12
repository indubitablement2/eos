use super::*;

pub trait BattlescapeEventHandler {
    fn fleet_added(&mut self, bc: &Battlescape, fleet_id: FleetId);
    fn ship_destroyed(&mut self, fleet_id: FleetId, index: usize);
    fn entity_removed(&mut self, entity_id: EntityId, entity: Entity);
    fn entity_added(&mut self, entity_id: EntityId, entity: &Entity);
    /// Calling step after this event is emitted will have no effect.
    fn battle_over(&mut self, bc: &Battlescape);
}

impl BattlescapeEventHandler for () {
    fn fleet_added(&mut self, _bc: &Battlescape, _fleet_id: FleetId) {}
    fn ship_destroyed(&mut self, _fleet_id: FleetId, _index: usize) {}
    fn entity_removed(&mut self, _entity_id: EntityId, _entity: Entity) {}
    fn entity_added(&mut self, _entity_id: EntityId, _entity: &Entity) {}
    fn battle_over(&mut self, _bc: &Battlescape) {}
}

// SERVER
// manager
// connections

// MANAGER
// simulations: [simulation],
// factions

// BUYING SHIP
// Client send buy order to its manager (buy x credit worth of y at z)
// manager relay to main
// main handle buy order (freeze assets). If successful send buy order cmd to manager
// manager relay cmd to sim
// sim change ship owner and emit ship owner change event
// manager relay event to main
// main keep track of client ships and notify client of changes if connected

// BUYING GOODS
// Client send buy order
// main handle buy order. If successful send goods owner change cmd to manager
// manager relay cmd to sim
// sim moves goods from cargo to cargo

// SELLING
//

// MANUFACTURING
//

// INCOME
// sim emit income event
// manager relay to main
// main keep track of income over time
// main notify client of income every few sec if any and is connected
