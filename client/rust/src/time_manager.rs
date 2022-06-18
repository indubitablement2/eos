use common::timef::TimeF;
use utils::ring_buffer::*;
use common::TICK_DURATION;

#[derive(Debug, Clone, Copy)]
pub struct TimeManagerConfigs {
    /// Amount of a period with an empty buffer to trigger a target buffer size increase. `0..1`
    pub threshold: f32,

    /// Amount of real time in each period.
    /// If we have a full period without any empty buffer, we will try to decrease the target buffer size.
    pub period: f32,

    /// We will hard catch up if the buffer size is above this.
    pub max_buffer_size: u32,
    /// Target buffer will never go under this.
    pub min_target_buffer: u32,
}
impl Default for TimeManagerConfigs {
    fn default() -> Self {
        Self { threshold: 0.15, period: 10.0, max_buffer_size: 10, min_target_buffer: 1 }
    }
}

pub struct TimeManager {
    pub max_tick: u32,
    pub tick: u32,
    /// Factionnal part of a tick.
    pub tick_frac: f32,

    current_period: f32,
    num_empty: f32,

    pub time_dilation: f32,
    pub recent_time_dilation: RingBufferF<10>,
    pub target_buffer: u32,

    pub configs: TimeManagerConfigs,
}
impl TimeManager {
    pub fn new(configs: TimeManagerConfigs) -> Self {
        Self {
            max_tick: 0,
            tick: 0,
            tick_frac: 0.0,
            current_period: 0.0,
            num_empty: 0.0,
            time_dilation: 1.0,
            recent_time_dilation: RingBufferF::new(1.0),
            target_buffer: configs.min_target_buffer,
            configs,
        }
    }

    pub fn maybe_max_tick(&mut self, new_tick: u32) {
        self.max_tick = self.max_tick.max(new_tick);
    }

    pub fn update(&mut self, real_delta: f32) {
        self.tick_frac += real_delta * self.time_dilation;

        let advance = (self.tick_frac / TICK_DURATION.as_secs_f32()) as u32;

        let buffer_size = match self.max_tick.checked_sub(self.tick) {
            Some(buffer_size) => {
                self.tick += advance;
                self.tick_frac -= advance as f32 * TICK_DURATION.as_secs_f32();
                buffer_size
            },
            None => {
                self.tick_frac = TICK_DURATION.as_secs_f32() * 0.99;
                0
            },
        };

        // Hard catch up if we are too far behind.
        if buffer_size > self.configs.max_buffer_size {
            self.tick += buffer_size - 1;
            self.time_dilation = 1.0;
            self.recent_time_dilation.buffer.fill(1.0);
            self.target_buffer = self.configs.min_target_buffer;
            self.new_period();
            return;
        }

        if buffer_size == 0 {
            self.num_empty += real_delta / self.configs.period;
        }

        // Change target buffer size.
        if self.num_empty > self.configs.threshold {
            // We have seen too many empty buffer, so we increase the target buffer size.
            self.target_buffer = (self.target_buffer + 1).min(self.configs.max_buffer_size);
            self.new_period();
        } else if self.current_period >= self.configs.period {
            if self.num_empty == 0.0 {
                // We have not seen any empty buffer, so we decrease the target buffer size.
                self.target_buffer = self.target_buffer.saturating_sub(1).max(self.configs.min_target_buffer);
            }
            self.new_period();
        }

        // Change time dilation to reach the target buffer.
        let new_time_dilation = if buffer_size > self.target_buffer {
            // Speed up time. 
            (buffer_size as f32 - self.target_buffer as f32).mul_add(0.02, 1.0)
        } else {
            // Slow down time or go toward neutral time dilation.
            (self.target_buffer as f32 - buffer_size as f32).mul_add(0.02, 1.0)
        };
        self.recent_time_dilation.set_next(new_time_dilation);
        self.time_dilation = self.recent_time_dilation.buffer.iter().fold(0.0, |i, t| i+t) / self.recent_time_dilation.buffer.len() as f32;
    }

    pub fn orbit_time(&self) -> f32 {
        TimeF{ tick: self.tick, tick_frac: self.tick_frac }.to_orbit_time()
    }

    fn new_period(&mut self) {
        self.current_period = 0.0;
        self.num_empty = 0.0;
    }
}