use super::*;

pub struct Interval {
    internal_time: Instant,
    max_difference: Duration,
    target_interval: Duration,
}
impl Interval {
    pub fn new(interval: u64, max_difference: u64) -> Self {
        Self {
            internal_time: Instant::now(),
            max_difference: Duration::from_millis(max_difference),
            target_interval: Duration::from_millis(interval),
        }
    }

    pub fn step(&mut self) {
        let now = Instant::now();

        self.internal_time += self.target_interval;

        let behind = now - self.internal_time;
        if behind > self.max_difference {
            log::debug!(
                "Interval behind by {}ms which is more than maximum of {}ms",
                behind.as_millis(),
                self.max_difference.as_millis()
            );
            self.internal_time = now - self.max_difference;
        }

        if let Some(delay) = self.internal_time.checked_duration_since(now) {
            std::thread::sleep(delay);
        }
    }
}
