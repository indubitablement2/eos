use super::*;
use crate::metascape::{fleet::Fleet, ship::Ship};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeFleet {
    pub owner: Option<ClientId>,
    pub ships: Vec<BattlescapeFleetShip>,
    pub team: u32,
}
impl BattlescapeFleet {
    pub fn from_fleet(fleet: Fleet, team: u32) -> Self {
        Self {
            owner: fleet.owner,
            ships: fleet
                .ships
                .into_iter()
                .map(|ship| BattlescapeFleetShip::new(ship))
                .collect(),
            team,
        }
    }

    pub fn try_spawn(&mut self, ship_idx: usize) -> Option<(EntityCondition, EntityDataId)> {
        if let Some(ship) = self.ships.get_mut(ship_idx) {
            if let FleetShipState::Ready = ship.state {
                ship.state = FleetShipState::Spawned;
                Some((
                    ship.condition,
                    ship.original_ship.ship_data_id.data().entity_data_id,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    // /// Can fail if the ship is already spawned or does not exist.
    // pub fn try_spawn(
    //     &mut self,
    //     fleet_id: FleetId,
    //     ship_idx: usize,
    //     spawn_point: &mode::SpawnPoint,
    //     bs_half_size: f32,
    //     entity_id: EntityId,
    //     physics: &mut Physics,
    // ) -> Option<Entity> {
    //     self.ships.get_mut(ship_idx).and_then(|ship| {
    //         if ship.state.is_ready() {
    //             ship.state = FleetShipState::Spawned;
    //         } else {
    //             None
    //         }
    //     })

    //     if let Some(fleet_ship) = self.ships.get_mut(ship_idx) {
    //         if !fleet_ship.state.is_ready() {
    //             return None;
    //         }
    //         fleet_ship.state = FleetShipState::Spawned;

    //         // Find spawn point.
    //         let angle = spawn_point.spawn_direction_angle;
    //         let mut translation = spawn_point.spawn_position * bs_half_size;
    //         let mut spawn_iso = na::Isometry2::new(translation, angle);
    //         let y_offset = Rotation::new(angle)
    //             .transform_vector(&na::vector![0.0, -Battlescape::SPAWN_OFFSET]);
    //         let x_offset = Rotation::new(FRAC_PI_2).transform_vector(&y_offset);
    //         let test_shape = Ball::new(Battlescape::SPAWN_OFFSET);
    //         let mut total_y_offset = na::Vector2::zeros();
    //         'outer: loop {
    //             for x in -3..4 {
    //                 spawn_iso.translation.vector =
    //                     translation + (x as f32 * x_offset) + total_y_offset;
    //                 if physics
    //                     .intersection_any_with_shape(&spawn_iso, &test_shape, QueryFilter::new())
    //                     .is_none()
    //                 {
    //                     // Can spawn here without collision.
    //                     translation = spawn_iso.translation.vector;
    //                     break 'outer;
    //                 }
    //             }
    //             total_y_offset += y_offset;
    //         }

    //         Some(Entity::new(
    //             fleet_ship.original_ship.ship_data_id.data().entity_data_id,
    //             entity_id,
    //             physics,
    //             translation,
    //             angle,
    //             Some((fleet_id, ship_idx)),
    //             self.team,
    //         ))
    //     } else {
    //         None
    //     }
    // }

    // pub fn ship_removed(&mut self, ship_idx: usize, condition: EntityCondition) {
    //     self.ships[ship_idx].state = FleetShipState::Removed(condition);
    // }

    // pub fn ship_destroyed(&mut self, ship_idx: usize) {
    //     self.ships[ship_idx].state = FleetShipState::Destroyed;
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeFleetShip {
    pub original_ship: Ship,
    pub condition: EntityCondition,
    pub state: FleetShipState,
}
impl BattlescapeFleetShip {
    pub fn new(ship: Ship) -> Self {
        Self {
            condition: ship.condition,
            original_ship: ship,
            state: FleetShipState::Ready,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FleetShipState {
    Ready,
    Spawned, // TODO: Delay before hable to retreat.
    Removed, // TODO: Delay before hable to re-enter.
    Destroyed,
}
