use super::*;

pub struct Interval {
    previous_step: Instant,
    target_interval: Duration,
}
impl Interval {
    pub fn new(millis: u64) -> Self {
        Self {
            previous_step: Instant::now(),
            target_interval: Duration::from_millis(millis),
        }
    }

    pub fn step(&mut self) {
        if let Some(remaining) = self
            .target_interval
            .checked_sub(self.previous_step.elapsed())
        {
            std::thread::sleep(remaining);
        }

        self.previous_step = Instant::now();
    }
}
