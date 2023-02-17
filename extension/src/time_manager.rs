use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeManagerConfig {
    /// Amount of real time in each period.
    /// Time dilation is updated by using the performance from the previous period.
    pub period: f32,
    /// Time dilation will never stray away from 1 more than this.
    pub max_time_change: f32,
    /// We will hard catch up if the buffered time is above this.
    pub max_buffer: f32,
    /// We will hard catch up if the buffered time is under this.
    pub min_buffer: f32,
    /// Amount of buffer we try to reach by changing time dilation.
    pub wish_buffer: f32,
    /// Multiply time dilation strenght when speeding up time.
    /// Something low avoid overshoot.
    pub increase_change_strenght: f32,
    /// Multiply time dilation strenght when slowing down time.
    pub decrease_change_strenght: f32,
}
impl TimeManagerConfig {
    /// Minimim amount of buffering.
    pub fn local() -> Self {
        Self {
            max_buffer: 0.3,
            min_buffer: 0.0,
            wish_buffer: 0.15,
            ..Default::default()
        }
    }

    pub fn very_smooth() -> Self {
        Self {
            max_buffer: 0.8,
            min_buffer: 0.0,
            wish_buffer: 0.4,
            ..Default::default()
        }
    }
}
impl Default for TimeManagerConfig {
    fn default() -> Self {
        Self {
            period: 1.0,
            max_time_change: 0.08,
            max_buffer: 0.45,
            min_buffer: 0.0,
            wish_buffer: 0.25,
            increase_change_strenght: 0.6,
            decrease_change_strenght: 0.8,
        }
    }
}

/// F: tick duration in milliseconds.
pub struct TimeManager<const F: u32> {
    pub max_tick: u64,
    /// The current tick for the simulation state.
    ///
    /// The tick we are interpolating toward (see `tick_frac`) for rendering.
    ///
    /// Will never be more than `max_tick`.
    pub tick: u64,
    /// Fraction of a tick in seconds.
    /// Used for rendering interpolation.
    /// Counted from tick - 1.
    /// This could be more than a tick if we are tick starved.
    pub tick_frac: f32,

    pub current_period: f32,
    pub min_over_period: f32,
    pub time_dilation: f32,

    pub config: TimeManagerConfig,
}
impl<const F: u32> TimeManager<F> {
    const TICK_DURATION: f32 = F as f32 / 1000.0;

    pub fn new(config: TimeManagerConfig) -> Self {
        Self {
            max_tick: 0,
            tick: 0,
            tick_frac: 0.0,
            current_period: 0.0,
            config,
            min_over_period: 10.0,
            time_dilation: 1.0,
        }
    }

    /// We need to know the last available tick.
    ///
    /// Call this every time you get a new tick ready.
    pub fn maybe_max_tick(&mut self, new_tick: u64) {
        self.max_tick = self.max_tick.max(new_tick);
    }

    pub fn reset(&mut self) {
        let mut s = Self {
            config: self.config,
            ..Default::default()
        };
        std::mem::swap(self, &mut s);
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

        // Hard catch up if we are too far behind or ahead.
        if remaining > self.config.max_buffer {
            let buffer_size = self.max_tick - self.tick;
            self.tick += buffer_size - 1;
            self.tick_frac = 0.0;
            self.time_dilation = 1.0;
            self.new_period();

            log::info!(
                "Buffer time ({:.2}) over limit of {}. Catching up...",
                buffer_size as f32 * Self::TICK_DURATION,
                self.config.max_buffer
            );
            return true;
        } else if remaining < self.config.min_buffer {
            // The minimum amount of time change to not be ahead.
            let change = remaining - self.config.min_buffer;
            log::info!(
                "Buffer time ({:.4}) under limit of {}. Modifying time by up to {:.4}...",
                remaining,
                self.config.min_buffer,
                change
            );
            self.tick_frac = 0.0f32.max(self.tick_frac + change);
        }

        if remaining < self.config.wish_buffer && self.time_dilation > 1.0 {
            self.time_dilation = 1.0;
        }

        // Compute new time dilation.
        if self.current_period >= self.config.period {
            let mut time_change = (self.min_over_period - self.config.wish_buffer)
                .clamp(-self.config.max_time_change, self.config.max_time_change);

            if time_change > 0.0 {
                time_change *= self.config.increase_change_strenght;
            } else {
                time_change *= self.config.decrease_change_strenght;
            }

            self.time_dilation = (time_change + self.config.period) / self.config.period;

            self.new_period();
        }

        self.tick != previous_tick
    }

    /// How many seconds of tick buffer remaining.
    pub fn buffer_time_remaining(&self) -> f32 {
        (self.max_tick - self.tick + 1) as f32 * Self::TICK_DURATION - self.tick_frac
    }

    /// Used for rendering.
    /// ## Panic:
    /// - `tick_start` > `tick_end`
    /// - `tick_start` > `tick`
    pub fn compute_interpolation(&self, tick_start: u64, tick_end: u64) -> f32 {
        if let Some(range) = tick_end.checked_sub(tick_start) {
            let elapsed =
                (self.tick - tick_start) as f32 - 1.0 + self.tick_frac / Self::TICK_DURATION;
            elapsed / range as f32
        } else {
            0.0
        }
    }

    /// Used for rendering.
    ///
    /// Return how far we are from last tick (0.0) to current tick (1.0).
    pub fn interpolation_weight(&self) -> f32 {
        self.tick_frac / Self::TICK_DURATION
    }

    fn new_period(&mut self) {
        self.current_period = 0.0;
        self.min_over_period = 10.0;
    }
}

impl<const F: u32> Default for TimeManager<F> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
