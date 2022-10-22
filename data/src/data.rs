use super::*;
use crate::hull::*;
use crate::ship::*;

pub const DATA: Data = Data {
    hulls: &[
        // 0
        HullData {
            defence: Defence {
                hull: 100,
                armor: 100,
            },
            shape: HullShape::Ball { radius: 1.0 },
            density: 1.0,
        },
        // 1
        HullData {
            defence: Defence {
                hull: 100,
                armor: 100,
            },
            shape: HullShape::Ball { radius: 1.0 },
            density: 1.0,
        },
    ],
    ships: &[
        // 0
        ShipData {
            mobility: Mobility {
                linear_acceleration: 1.0,
                angular_acceleration: 1.0,
                max_linear_velocity: 1.0,
                max_angular_velocity: 1.0,
            },
            hulls: &[BcHullDataId(0)],
        },
        // 1
        ShipData {
            mobility: Mobility {
                linear_acceleration: 1.0,
                angular_acceleration: 1.0,
                max_linear_velocity: 1.0,
                max_angular_velocity: 1.0,
            },
            hulls: &[BcHullDataId(0)],
        },
    ],
};

pub struct Data {
    pub hulls: &'static [HullData],
    pub ships: &'static [ShipData],
}
