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

    /// Can fail if the ship is already spawned or does not exist.
    pub fn try_spawn(
        &mut self,
        ship_idx: usize,
        spawn_point: &mode::SpawnPoint,
        bc_half_size: f32,
        entity_id: EntityId,
        physics: &mut Physics,
    ) -> Option<Entity> {
        if let Some(fleet_ship) = self.ships.get_mut(ship_idx) {
            if !fleet_ship.state.is_ready() {
                return None;
            }
            fleet_ship.state = FleetShipState::Spawned;

            // Find spawn point.
            let angle = spawn_point.spawn_direction_angle;
            let mut translation = spawn_point.spawn_position * bc_half_size;
            let mut spawn_iso = na::Isometry2::new(translation, angle);
            let y_offset = Rotation::new(angle)
                .transform_vector(&na::vector![0.0, -Battlescape::SPAWN_OFFSET]);
            let x_offset = Rotation::new(FRAC_PI_2).transform_vector(&y_offset);
            let test_shape = Ball::new(Battlescape::SPAWN_OFFSET);
            let mut total_y_offset = na::Vector2::zeros();
            'outer: loop {
                for x in -3..4 {
                    spawn_iso.translation.vector =
                        translation + (x as f32 * x_offset) + total_y_offset;
                    if physics
                        .intersection_any_with_shape(&spawn_iso, &test_shape, QueryFilter::new())
                        .is_none()
                    {
                        // Can spawn here without collision.
                        translation = spawn_iso.translation.vector;
                        break 'outer;
                    }
                }
                total_y_offset += y_offset;
            }

            Some(Entity::new(
                fleet_ship.original_ship.ship_data_id.data().entity_data_id,
                entity_id,
                physics,
                translation,
                angle,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattlescapeFleetShip {
    pub original_ship: Ship,
    pub state: FleetShipState,
}
impl BattlescapeFleetShip {
    pub fn new(ship: Ship) -> Self {
        Self {
            original_ship: ship,
            state: FleetShipState::Ready,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FleetShipState {
    Ready,
    Spawned,               // TODO: Delay before hable to retreat.
    Removed(EntityResult), // TODO: Delay before hable to re-enter.
    Destroyed,
}
impl FleetShipState {
    pub fn is_ready(&self) -> bool {
        match self {
            FleetShipState::Ready => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntityResult {
    pub new_hull: f32,
    pub new_armor: f32,
    pub new_readiness: f32,
}
