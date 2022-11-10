#![feature(variant_count)]

pub mod hull_data;
pub mod ship_data;
pub mod id;
pub mod fleet;

use serde::{Deserialize, Serialize};

pub use ship_data::ShipDataId;
pub use hull_data::HullDataId;
pub use id::*;

extern crate nalgebra as na;
