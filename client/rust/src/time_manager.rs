use common::timef::TimeF;
use common::TICK_DURATION;

#[derive(Debug, Clone, Copy)]
pub struct TimeManagerConfigs {
    /// Amount of real time in each period.
    /// If we have a full period without any empty buffer, we will try to decrease the target buffer size.
    pub period: f32,
    /// We will hard catch up if the buffer size is above this.
    pub max_buffer_size: u32,
    /// Amount of buffer left we should try to not go under.
    pub wish_min_buffer: f32,
    /// Multiply the final time change amount when adding time.
    pub add_time_change_strenght: f32,
    /// Multiply the final time change amount when removing time.
    pub sub_time_change_strenght: f32,
    /// How much time (forward/backward) change can occur over one period.
    pub max_time_change: f32,
}
impl Default for TimeManagerConfigs {
    fn default() -> Self {
        Self {
            period: 10.0,
            max_time_change: 0.01,
            max_buffer_size: 10,
            wish_min_buffer: 0.01,
            add_time_change_strenght: 0.1,
            sub_time_change_strenght: 1.0,
        }
    }
}

/// - `tick`: The last consumed tick.
/// - `tick_frac`: Fraction of a tick. Counted from tick - 1.
///
/// ## Example:
/// `total_time = (tick - 1) * tick_duration + tick_frac`
pub struct TimeManager {
    pub max_tick: u32,
    /// The current tick for the simulation state.
    ///
    /// The tick we are interpolating toward (see `tick_frac`) for rendering.
    pub tick: u32,
    /// Used for rendering interpolation.
    /// Counted from tick - 1.
    /// This could be more than a tick if we are tick starved.
    pub tick_frac: f32,

    pub current_period: f32,
    pub min_over_period: f32,
    /// Amount of time change spread over this period.
    pub time_change_period: f32,

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
            time_change_period: 0.0,
        }
    }

    pub fn maybe_max_tick(&mut self, new_tick: u32) {
        self.max_tick = self.max_tick.max(new_tick);
    }

    pub fn update(&mut self, real_delta: f32) {
        self.current_period += real_delta;

        self.tick_frac += self
            .time_change_period
            .mul_add(real_delta / self.configs.period, real_delta);

        let wish_advance = (self.tick_frac / TICK_DURATION.as_secs_f32()) as u32;
        let max_advance = self.max_tick - self.tick;
        let advance = wish_advance.min(max_advance);

        self.tick += advance;
        self.tick_frac -= advance as f32 * TICK_DURATION.as_secs_f32();

        let buffer_size = self.max_tick - self.tick;

        // Hard catch up if we are too far behind.
        if buffer_size > self.configs.max_buffer_size {
            self.tick += buffer_size - 1;
            self.tick_frac = 0.0;
            self.new_period();
            log::info!(
                "Tick buffer size ({}) over limit of {}. Catching up...",
                buffer_size,
                self.configs.max_buffer_size
            );
            return;
        }

        let remaining = self.buffer_time_remaining();
        self.min_over_period = self.min_over_period.min(remaining);

        // Stop accelerating time if we have no buffer remaining.
        if self.time_change_period > 0.0 && remaining < self.configs.wish_min_buffer {
            self.time_change_period = 0.0;
        }

        if self.current_period >= self.configs.period {
            self.time_change_period = (self.min_over_period - self.configs.wish_min_buffer)
                .clamp(-self.configs.max_time_change, self.configs.max_time_change);

            if self.time_change_period > 0.0 {
                self.time_change_period *= self.configs.add_time_change_strenght;
            } else {
                self.time_change_period *= self.configs.sub_time_change_strenght;
            }
            // } else if self.max_over_period
            // self.time_change_period = self.max_over_period.clamp(-self.configs.max_time_change, self.configs.max_time_change);
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
