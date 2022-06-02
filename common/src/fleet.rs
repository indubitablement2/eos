use crate::{
    data,
    idx::{ShipBaseId, WeaponBaseId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipInfos {
    pub ship_base: ShipBaseId,
    /// 0..=1.0
    pub hp: f32,
    /// 0..=1.0
    pub state: f32,
    pub weapon_bases: Vec<WeaponBaseId>,
}

/// The stats derived from the fleet's ships.
#[derive(Debug, Clone, Copy)]
pub struct FleetStats {
    /// How much velocity this fleet can gain each update.
    pub acceleration: f32,
    /// How much space this fleet takes.
    pub radius: f32,
    /// Extra radius this fleet will get detected.
    pub detected_radius: f32,
    /// Radius this fleet will detect things.
    pub detector_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetComposition {
    pub ships: Vec<ShipInfos>,
}
impl FleetComposition {
    pub fn compute_auto_combat_strenght(&self) -> f32 {
        let data = data();
        self.ships.iter().fold(0.0, |acc, ship_infos| {
            acc + data.ships[ship_infos.ship_base].auto_combat_strenght * ship_infos.state
        })
    }

    pub fn compute_stats(&self) -> FleetStats {
        // TODO: Compute fleet's stats
        FleetStats {
            acceleration: 0.04,
            radius: self.ships.len() as f32 * 0.1,
            detected_radius: 10.0,
            detector_radius: 10.0,
        }
    }

    #[deprecated]
    pub fn new_debug() -> Self {
        use crate::*;
        use rand::prelude::*;
        let mut rng = thread_rng();

        let num_ship = data().ships.len() as u32;

        let ships = (0..rng.gen_range(1..10))
            .map(|_| ShipInfos {
                ship_base: ShipBaseId::from_raw(rng.gen_range(0..num_ship)),
                hp: 1.0,
                state: 1.0,
                weapon_bases: Default::default(),
            })
            .collect();

        Self { ships }
    }
}
