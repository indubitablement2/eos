use std::{time::{Duration, Instant}, thread::sleep};

/// Will sleep so that each call to tick are about interval duration apart.
/// 
/// ## Example:
/// interval is `100 ms`
/// - work `80 ms`
/// - sleep `20 ms`
/// - work `150 ms`
/// - sleep `0 ms` (we are over by `50 ms`)
/// - work `25 ms` 
/// - sleep `25 ms` (interval - (`50 ms` over from last tick + `25 ms`))
pub struct Interval {
    interval: Duration,
    last_tick: Instant,
    debt: Duration,
}
impl Interval {
    pub fn new(interval: Duration,) -> Self {
        Self { interval, last_tick: Instant::now(), debt: Duration::ZERO }
    }

    pub fn tick(&mut self) {
        let delta = self.last_tick.elapsed();
        if let Some(remaining) = self.interval.checked_sub(delta) {
            if let Some(sleep_dur) = remaining.checked_sub(self.debt) {
                sleep(sleep_dur);
            }
            self.debt = Duration::ZERO;
        } else {
            // Last tick took longer than interval. 
            // Next sleep (if any) will be shortened by the amount we are over.
            self.debt = delta - self.interval;
        }
        self.last_tick = Instant::now();
    }
}
