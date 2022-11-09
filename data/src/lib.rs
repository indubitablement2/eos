pub mod hull_data;
pub mod ship_data;

pub use ship_data::ShipDataId;
pub use hull_data::HullDataId;
use serde::{Deserialize, Serialize};

extern crate nalgebra as na;
