use super::*;

/// Takes up 2^16 * 2 bytes of memory.
#[derive(Clone, Serialize, Deserialize)]
pub struct SmallIdDispenser {
    free: std::collections::VecDeque<u16>,
}
impl SmallIdDispenser {
    pub fn new_id(&mut self) -> Option<u16> {
        self.free.pop_front()
    }

    pub fn delete_id(&mut self, id: u16) {
        self.free.push_back(id)
    }
}
impl Default for SmallIdDispenser {
    fn default() -> Self {
        Self {
            free: (0..u16::MAX).collect(),
        }
    }
}
