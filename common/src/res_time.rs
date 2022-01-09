#[derive(Debug, Clone, Copy)]
pub struct TimeRes {
    pub tick: u64,
}
impl TimeRes {
    pub fn as_time(self) -> f32 {
        (self.tick % u32::MAX as u64) as f32
    }
}
impl Default for TimeRes {
    fn default() -> Self {
        Self { tick: 0 }
    }
}
