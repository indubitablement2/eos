use super::*;
use once_cell::sync::Lazy;

pub static BATTLESCAPE_DATA: Lazy<BattlescapeData> = Lazy::new(|| BattlescapeData {
    hulls: vec![
        // 0
        HullData {
            defence: Defence {
                hull: 100,
                armor: 100,
            },
            shape: HullShape::Ball { radius: 4.0 },
            density: 1.0,
        },
        // 1
        HullData {
            defence: Defence {
                hull: 100,
                armor: 100,
            },
            shape: HullShape::Ball { radius: 4.0 },
            density: 1.0,
        },
    ],
    ships: vec![
        // 0
        ShipData {
            mobility: Mobility {
                linear_acceleration: 1.0,
                angular_acceleration: 1.0,
                max_linear_velocity: 1.0,
                max_angular_velocity: 1.0,
            },
            hulls_data_index: smallvec![0],
        },
        // 1
        ShipData {
            mobility: Mobility {
                linear_acceleration: 1.0,
                angular_acceleration: 1.0,
                max_linear_velocity: 1.0,
                max_angular_velocity: 1.0,
            },
            hulls_data_index: smallvec![0],
        },
    ],
});

pub struct BattlescapeData {
    pub hulls: Vec<HullData>,
    pub ships: Vec<ShipData>,
}
