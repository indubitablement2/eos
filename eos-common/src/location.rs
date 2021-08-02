use crate::const_var::*;
use crate::idx::SectorId;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::ops::Sub;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
/// Location on the SpaceGrid.
pub struct Location {
    pub sector_id: SectorId,
    pub local_position: Vec2,
}

impl Sub for Location {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            sector_id: SectorId(self.sector_id.0 - rhs.sector_id.0),
            local_position: self.local_position - rhs.local_position,
        }
    }
}
impl Location {
    /// Return the shortest direction to another location. Not accurate, because of float error.
    pub fn euclid_direction(&self, loc_to: Location) -> Vec2 {
        let mut distance = loc_to.local_position - self.local_position;

        let vec_dif = (loc_to.sector_id.to_ivec() - self.sector_id.to_ivec()).as_f32();

        distance.x += vec_dif.x * SECTOR_SIZE;
        distance.y += vec_dif.y * SECTOR_SIZE;

        distance
    }
}
