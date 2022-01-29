use common::UPDATE_INTERVAL;
use std::collections::VecDeque;

/// Metrics in seconds.
pub struct PerformanceMetrics {
    pub recents: VecDeque<f32>,
    pub weighted_average: f32,
}
impl PerformanceMetrics {
    pub const TARGET_NUM_RECENT: usize = 100;
    pub const BUDGET: f32 = UPDATE_INTERVAL.as_secs_f32();
    /// When computing weighted average, this is how much of the past value is kept.
    const AVERAGE_WEIGHT: f32 = 0.95;

    pub fn update(&mut self, elapsed: f32) {
        self.recents.push_front(elapsed);
        self.recents.truncate(Self::TARGET_NUM_RECENT);

        self.weighted_average = self
            .weighted_average
            .mul_add(Self::AVERAGE_WEIGHT, elapsed * (1.0 - Self::AVERAGE_WEIGHT));
    }
}
impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            recents: Default::default(),
            weighted_average: Self::BUDGET,
        }
    }
}
