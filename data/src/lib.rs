#![feature(const_slice_index)]

pub mod hull_data;

use hull_data::*;
use serde::{Deserialize, Serialize};

extern crate nalgebra as na;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct HullDataId(pub u32);
#[inline]
pub const fn get_hull_data(id: HullDataId) -> Option<&'static HullData> {
    HULLS.get(id.0 as usize)
}
#[inline]
pub const fn hull_data(id: HullDataId) -> &'static HullData {
    &HULLS[id.0 as usize]
}
