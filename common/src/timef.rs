use crate::orbit::ORBIT_TICK_PERIOD;
use crate::TICK_DURATION;

#[derive(Debug, Clone, Copy)]
pub struct TimeF {
    /// Number of whole tick.
    pub total_tick: u64,
    /// Fraction of a tick in seconds.
    ///
    /// Range from 0 to `TICK_DURATION` as seconds exclusive.
    pub tick_frac: f32,
}
impl TimeF {
    pub fn tick_to_orbit_time(total_tick: u64) -> f32 {
        (total_tick % ORBIT_TICK_PERIOD) as f32 * TICK_DURATION.as_secs_f32()
    }

    pub fn to_orbit_time(&self) -> f32 {
        ((self.total_tick % ORBIT_TICK_PERIOD) as f32 * TICK_DURATION.as_secs_f32()) + self.tick_frac
    }
}
