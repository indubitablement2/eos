#![feature(const_slice_index)]

pub mod data;
pub mod hull;
pub mod ship;

use self::data::*;
use hull::HullData;
use serde::{Deserialize, Serialize};
use ship::ShipData;

extern crate nalgebra as na;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BcHullDataId(pub u32);
#[inline]
pub const fn get_bc_hull_data(id: BcHullDataId) -> Option<&'static HullData> {
    DATA.hulls.get(id.0 as usize)
}
#[inline]
pub const fn bc_hull_data(id: BcHullDataId) -> &'static HullData {
    &DATA.hulls[id.0 as usize]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ShipDataId(pub u32);
#[inline]
pub const fn get_ship_data(id: ShipDataId) -> Option<&'static ShipData> {
    DATA.ships.get(id.0 as usize)
}
#[inline]
pub const fn ship_data(id: ShipDataId) -> &'static ShipData {
    &DATA.ships[id.0 as usize]
}
