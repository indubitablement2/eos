#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Time {
    pub tick: u32,
    pub total_tick: u64,
}
impl Time {
    pub fn increment(&mut self) {
        self.tick += 1;
        self.total_tick += 1;
    }

    pub fn as_time(self) -> f32 {
        self.tick as f32
    }
}
