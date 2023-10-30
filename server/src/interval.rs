use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct Interval {
    interval: Duration,
    last: Instant,
}
impl Interval {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: Instant::now(),
        }
    }

    /// Sleep until the duration between the previous
    /// call to `step` is at least the interval.
    pub fn step(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last;
        self.last = now;

        if let Some(remaining) = self.interval.checked_sub(elapsed) {
            sleep(remaining);
        }
    }
}

#[test]
fn test_interval() {
    let duration = Duration::from_millis(10);
    let mut interval = Interval::new(duration);
    let mut now = Instant::now();

    interval.step();
    assert!(now.elapsed() >= duration);
    assert!(now.elapsed() < duration + Duration::from_millis(1));

    now = Instant::now();
    sleep(duration + Duration::from_millis(1));
    interval.step();
    assert!(now.elapsed() >= duration);
    assert!(now.elapsed() < duration + Duration::from_millis(2));

    now = Instant::now();
    sleep(duration - Duration::from_millis(1));
    interval.step();
    assert!(now.elapsed() >= duration);
    assert!(now.elapsed() < duration + Duration::from_millis(1));
}
