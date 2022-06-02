use super::*;

pub struct KnowFleets {
    pub fleets: Vec<(FleetId, u16)>,
    next_small_id: u16,
    recycled_small_id: (Vec<u16>, Vec<u16>),
}
impl Default for KnowFleets {
    fn default() -> Self {
        Self {
            fleets: Default::default(),
            next_small_id: 1,
            recycled_small_id: Default::default(),
        }
    }
}
impl KnowFleets {
    pub fn get_new_small_id(&mut self) -> u16 {
        self.recycled_small_id.0.pop().unwrap_or_else(|| {
            let next = self.next_small_id;
            self.next_small_id += 1;
            next
        })
    }

    pub fn reuse_small_id(&mut self) {
        self.recycled_small_id
            .0
            .append(&mut self.recycled_small_id.1);
    }

    pub fn recycle_small_id(&mut self, small_id: u16) {
        self.recycled_small_id.1.push(small_id);
    }
}
