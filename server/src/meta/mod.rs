pub mod fleet;
mod recyclable_raw_table;

use common::time::Time;
use soak::RawTable;

use self::fleet::Fleet;

pub struct FleetComponents {
    // raw_table: recyclable_raw_table::RecyclableRawTable<RawTable<Fleet>, Fleet>,
}

pub struct Metascape {
    time: Time,

    pub fleet_components: RawTable<Fleet>,
    /// How many fleets are in `fleet_components`.
    pub fleet_last_index: usize,
    /// Index that are ready to be reused.
    pub fleet_removed: Vec<usize>,
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
