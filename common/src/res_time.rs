#[derive(Debug, Clone, Copy)]
pub struct TimeRes {
    pub tick: u32,
    pub total_tick: u64,
}
impl TimeRes {
    pub fn increment(&mut self) {
        self.tick += 1;
        self.total_tick += 1;
    }

    pub fn as_time(self) -> f32 {
        self.tick as f32
    }
}
impl Default for TimeRes {
    fn default() -> Self {
        Self { tick: 0, total_tick: 0 }
    }
}
