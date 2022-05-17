pub mod fleet;
mod recyclable_raw_table;

use self::{fleet::Fleet, recyclable_raw_table::Fleets};
use common::time::Time;
use soak::RawTable;

pub struct Metascape {
    time: Time,

    pub fleets: Fleets,
    pub fleet_queue_remove: (Vec<usize>, Vec<usize>),
}
impl Metascape {
    // unsafe fn asd(&mut self) {
    //     let mut position = self.fleet_components.ptr(FleetComponents::position);
    //     let p = position as usize;
    //     // let p2 = &mut *(p as *mut Vec2).add(i);

    //     for i in 0..self.fleet_components.capacity() {
    //         let position = unsafe{&mut *position.add(i)};
    //     }
    // }
}
