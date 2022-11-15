use super::*;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SpawnShip {
//     pub player_id: u16,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SetClientControl {
//     pub client_id: ClientId,
//     pub ship_id: Option<ShipId>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SetClientInput {
//     pub client_id: ClientId,
//     pub inputs: bc_client::PlayerInput,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AddFleet {
//     pub fleet_id: FleetId,
//     pub fleet: Fleet,
//     /// This should be an existing team.
//     /// If `None`, create a new team.
//     pub team: Option<Team>,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipMoveOrder {
    pub ship_id: Vec<ShipId>,
    pub position: na::Vector2<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    ShipMoveOrder(ShipMoveOrder),
}
