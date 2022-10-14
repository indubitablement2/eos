#[derive(Debug, Clone, Copy)]
pub struct TimeManagerConfigs {
    /// Amount of real time in each period.
    /// Time dilation is updated by using the performance from the previous period.
    pub period: f32,
    /// Time dilation will never stray away from 1 more than this.
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
            increase_change_strenght: 0.2,
            decrease_change_strenght: 1.0,
        }
    }
}

/// F: tick duration in milliseconds.
pub struct TimeManager<const F: u32> {
    pub max_tick: u64,
    /// The current tick for the simulation state.
    ///
    /// The tick we are interpolating toward (see `tick_frac`) for rendering.
    pub tick: u64,
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
impl<const F: u32> TimeManager<F> {
    /// Tick duration in seconds.
    const TICK_DURATION: f32 = F as f32 / 1000.0;

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

    /// We need to know the last available tick.
    ///
    /// Call this every time you get a new tick ready.
    pub fn maybe_max_tick(&mut self, new_tick: u64) {
        self.max_tick = self.max_tick.max(new_tick);
    }

    /// Return if the tick was incremented.
    pub fn update(&mut self, real_delta: f32) -> bool {
        let previous_tick = self.tick;

        self.current_period += real_delta;

        self.tick_frac += real_delta * self.time_dilation;

        let wish_advance = (self.tick_frac / Self::TICK_DURATION) as u64;
        let max_advance = self.max_tick - self.tick;
        let advance = wish_advance.min(max_advance);

        self.tick += advance;
        self.tick_frac -= advance as f32 * Self::TICK_DURATION;

        let remaining = self.buffer_time_remaining();
        self.min_over_period = self.min_over_period.min(remaining);

        // Hard catch up if we are too far behind.
        if remaining > self.configs.max_buffer {
            let buffer_size = self.max_tick - self.tick;
            self.tick += buffer_size - 1;
            self.tick_frac = 0.0;
            self.time_dilation = 1.0;
            self.new_period();
            log::info!(
                "Buffer time ({:.2}) over limit of {}. Catching up...",
                buffer_size as f32 * Self::TICK_DURATION,
                self.configs.max_buffer
            );
            return true;
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

        self.tick != previous_tick
    }

    /// How many seconds of tick buffer remaining.
    pub fn buffer_time_remaining(&self) -> f32 {
        (self.max_tick - self.tick + 1) as f32 * Self::TICK_DURATION - self.tick_frac
    }

    /// Used for rendering.
    ///
    /// Return the how far we are from last tick (0.0) to current tick (1.0).
    pub fn interpolation_weight(&self) -> f32 {
        self.tick_frac / Self::TICK_DURATION
    }

    fn new_period(&mut self) {
        self.current_period = 0.0;
        self.min_over_period = 10.0;
    }
}
