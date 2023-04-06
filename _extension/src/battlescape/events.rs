use super::*;
use crate::client_battlescape::ClientBattlescapeEventHandler;

pub trait BattlescapeEventHandlerTrait {
    /// Called once per step at the very end.
    fn stepped(&mut self, bs: &Battlescape);
    fn fleet_added(&mut self, fleet_id: FleetId);
    fn ship_state_changed(&mut self, fleet_id: FleetId, ship_idx: usize, state: FleetShipState);
    fn entity_removed(&mut self, entity_id: EntityId, entity: Entity);
    fn entity_added(&mut self, entity_id: EntityId, entity: &Entity, position: na::Isometry2<f32>);
    /// Calling step after this event is emitted will have no effect.
    fn battle_over(&mut self);
}

#[derive(Default)]
pub enum BattlescapeEventHandler {
    #[default]
    None,
    Client(ClientBattlescapeEventHandler),
    Server(()),
}
impl BattlescapeEventHandler {
    pub fn cast_client(self) -> Option<ClientBattlescapeEventHandler> {
        match self {
            BattlescapeEventHandler::None => None,
            BattlescapeEventHandler::Client(events) => Some(events),
            BattlescapeEventHandler::Server(_) => None,
        }
    }
}
impl BattlescapeEventHandlerTrait for BattlescapeEventHandler {
    fn stepped(&mut self, bs: &Battlescape) {
        match self {
            BattlescapeEventHandler::None => {}
            BattlescapeEventHandler::Client(events) => events.stepped(bs),
            BattlescapeEventHandler::Server(events) => events.stepped(bs),
        }
    }

    fn fleet_added(&mut self, fleet_id: FleetId) {
        match self {
            BattlescapeEventHandler::None => {}
            BattlescapeEventHandler::Client(events) => events.fleet_added(fleet_id),
            BattlescapeEventHandler::Server(events) => events.fleet_added(fleet_id),
        }
    }

    fn ship_state_changed(&mut self, fleet_id: FleetId, ship_index: usize, state: FleetShipState) {
        match self {
            BattlescapeEventHandler::None => {}
            BattlescapeEventHandler::Client(events) => {
                events.ship_state_changed(fleet_id, ship_index, state)
            }
            BattlescapeEventHandler::Server(events) => {
                events.ship_state_changed(fleet_id, ship_index, state)
            }
        }
    }

    fn entity_removed(&mut self, entity_id: EntityId, entity: Entity) {
        match self {
            BattlescapeEventHandler::None => {}
            BattlescapeEventHandler::Client(events) => events.entity_removed(entity_id, entity),
            BattlescapeEventHandler::Server(events) => events.entity_removed(entity_id, entity),
        }
    }

    fn entity_added(&mut self, entity_id: EntityId, entity: &Entity, position: na::Isometry2<f32>) {
        match self {
            BattlescapeEventHandler::None => {}
            BattlescapeEventHandler::Client(events) => {
                events.entity_added(entity_id, entity, position)
            }
            BattlescapeEventHandler::Server(events) => {
                events.entity_added(entity_id, entity, position)
            }
        }
    }

    fn battle_over(&mut self) {
        match self {
            BattlescapeEventHandler::None => {}
            BattlescapeEventHandler::Client(events) => events.battle_over(),
            BattlescapeEventHandler::Server(events) => events.battle_over(),
        }
    }
}

impl BattlescapeEventHandlerTrait for () {
    fn stepped(&mut self, _bs: &Battlescape) {}
    fn fleet_added(&mut self, _fleet_id: FleetId) {}
    fn ship_state_changed(&mut self, _fleet_id: FleetId, _ship_idx: usize, _state: FleetShipState) {
    }
    fn entity_removed(&mut self, _entity_id: EntityId, _entity: Entity) {}
    fn entity_added(
        &mut self,
        _entity_id: EntityId,
        _entity: &Entity,
        _position: na::Isometry2<f32>,
    ) {
    }
    fn battle_over(&mut self) {}
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
