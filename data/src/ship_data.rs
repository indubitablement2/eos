use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShipDataId {
    BallShip,
    CuboidShip,
}
impl ShipDataId {
    pub const fn data(self) -> ShipData {
         match self {
            Self::BallShip => ShipData {
                mobility: Mobility {
                    linear_acceleration: 1.0,
                    angular_acceleration: 1.0,
                    max_linear_velocity: 1.0,
                    max_angular_velocity: 1.0,
                },
                main_hull: HullDataId::Ball,
                auxiliary_hulls: &[],
            },
            Self::CuboidShip => ShipData {
                mobility: Mobility {
                    linear_acceleration: 1.0,
                    angular_acceleration: 1.0,
                    max_linear_velocity: 1.0,
                    max_angular_velocity: 1.0,
                },
                main_hull: HullDataId::Cuboid,
                auxiliary_hulls: &[],
            },
        }
    }
}

#[derive(Debug)]
pub struct ShipData {
    pub mobility: Mobility,
    pub main_hull: HullDataId,
    // TODO: Add: init_offset.
    pub auxiliary_hulls: &'static [HullDataId],
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Mobility {
    pub linear_acceleration: f32,
    pub angular_acceleration: f32,
    pub max_linear_velocity: f32,
    pub max_angular_velocity: f32,
}
