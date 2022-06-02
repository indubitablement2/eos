use std::ops::AddAssign;
use utils::Incrementable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Time {
    pub tick: u32,
    pub total_tick: u64,
}
impl Time {
    pub fn as_timef(self) -> f32 {
        self.tick as f32
    }
}
impl AddAssign for Time {
    fn add_assign(&mut self, rhs: Self) {
        self.tick += rhs.tick;
        self.total_tick += rhs.total_tick;
    }
}
impl Incrementable for Time {
    fn one() -> Self {
        Self { tick: 1, total_tick: 1 }
    }
}
