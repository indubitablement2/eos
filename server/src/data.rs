use super::*;
use crate::system::{entity::EntityData, hull::HullData};

pub struct Data {
    pub hulls: Box<[HullData]>,
    pub entities: Box<[EntityData]>,
    // pub ships: Vec<ShipData>,
}
impl Data {
    pub fn data() -> &'static Self {
        unsafe { DATA.as_ref().expect("data should be initialized") }
    }
}
static mut DATA: Option<Data> = None;
