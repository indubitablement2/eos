use common::timef::TimeF;
use common::TICK_DURATION;

#[derive(Debug, Clone, Copy)]
pub struct TimeManagerConfigs {
    /// Amount of real time in each period.
    /// Time dilation is updated by using the performance from the previous period.
    pub period: f32,
    /// How much total time change (forward/backward) can occur over one period.
    pub max_time_change: f32,
    /// We will hard catch up if the buffered time is above this.
    pub max_buffer: f32,
    /// We will hard catch up if the buffered time is under this.
    pub min_buffer: f32,
    /// Amount of buffer we try reach by changing time dilation.
    pub wish_buffer: f32,
    /// Multiply time dilation when speeding up time.
    pub increase_change_strenght: f32,
    /// Multiply time dilation when slowing down time.
    pub decrease_change_strenght: f32,
}
impl Default for TimeManagerConfigs {
    fn default() -> Self {
        Self {
            period: 4.0,
            max_time_change: 0.08,
            max_buffer: 1.0,
            min_buffer: -0.1,
            wish_buffer: 0.05,
            increase_change_strenght: 0.1,
            decrease_change_strenght: 1.0,
        }
    }
}

pub struct TimeManager {
    pub max_tick: u32,
    /// The current tick for the simulation state.
    ///
    /// The tick we are interpolating toward (see `tick_frac`) for rendering.
    pub tick: u32,
    /// Fraction of a tick in seconds.
    /// Used for rendering interpolation.
    /// Counted from tick - 1.
    /// This could be more than a tick if we are tick starved.
    pub tick_frac: f32,

    pub current_period: f32,
    pub min_over_period: f32,
    pub time_dilation: f32,

    pub configs: TimeManagerConfigs,
}
impl TimeManager {
    pub fn new(configs: TimeManagerConfigs) -> Self {
        Self {
            max_tick: 0,
            tick: 0,
            tick_frac: 0.0,
            current_period: 0.0,
            configs,
            min_over_period: f32::MAX,
            time_dilation: 1.0,
        }
    }

    pub fn maybe_max_tick(&mut self, new_tick: u32) {
        self.max_tick = self.max_tick.max(new_tick);
    }

    pub fn update(&mut self, real_delta: f32) {
        self.current_period += real_delta;

        self.tick_frac += real_delta * self.time_dilation;

        let wish_advance = (self.tick_frac / TICK_DURATION.as_secs_f32()) as u32;
        let max_advance = self.max_tick - self.tick;
        let advance = wish_advance.min(max_advance);

        self.tick += advance;
        self.tick_frac -= advance as f32 * TICK_DURATION.as_secs_f32();

        let remaining = self.buffer_time_remaining();
        self.min_over_period = self.min_over_period.min(remaining);

        // Hard catch up if we are too far behind.
        if remaining > self.configs.max_buffer {
            let buffer_size = self.max_tick - self.tick;
            self.tick += buffer_size - 1;
            self.tick_frac = 0.0;
            self.new_period();
            log::info!(
                "Buffer time ({:.2}) over limit of {}. Catching up...",
                buffer_size as f32 * TICK_DURATION.as_secs_f32(),
                self.configs.max_buffer
            );
            return;
        } else if remaining < self.configs.min_buffer {
            self.tick_frac = 0.0;
        }

        // Stop accelerating time if we have no buffer remaining.
        if self.time_dilation > 1.0 && remaining < self.configs.wish_buffer {
            self.time_dilation = 1.0;
        }

        // Compute new time dilation.
        if self.current_period >= self.configs.period {
            let mut time_change = (self.min_over_period - self.configs.wish_buffer)
                .clamp(-self.configs.max_time_change, self.configs.max_time_change);

            if time_change > 0.0 {
                time_change *= self.configs.increase_change_strenght;
            } else {
                time_change *= self.configs.decrease_change_strenght;
            }

            self.time_dilation = (time_change + self.configs.period) / self.configs.period;

            self.new_period();
        }
    }

    pub fn orbit_time(&self) -> f32 {
        TimeF {
            tick: self.tick,
            tick_frac: self.tick_frac,
        }
        .to_orbit_time()
    }

    /// How many seconds of tick buffer remaining.
    pub fn buffer_time_remaining(&self) -> f32 {
        (self.max_tick - self.tick + 1) as f32 * TICK_DURATION.as_secs_f32() - self.tick_frac
    }

    /// Used for rendering.
    /// ## Panic:
    /// - `range_tick_start` > `range_tick_end`
    /// - `range_tick_start` > `tick`
    pub fn compute_interpolation(&self, tick_start: u32, tick_end: u32) -> f32 {
        let range = (tick_end - tick_start) as f32;
        let elapsed = (self.tick - tick_start) as f32 - 1.0 + self.tick_frac / TICK_DURATION.as_secs_f32();
        elapsed / range
    }

    fn new_period(&mut self) {
        self.current_period = 0.0;
        self.min_over_period = 10.0;
    }
}
