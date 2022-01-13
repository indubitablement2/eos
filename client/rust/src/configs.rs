#[derive(Debug, Clone, Copy)]
pub struct Configs {
    /// How much should state updates be buffered.
    pub target_delta: f32,
    pub system_draw_distance: f32,
}
impl Default for Configs {
    fn default() -> Self {
        Self {
            target_delta: 3.0,
            system_draw_distance: 256.0,
        }
    }
}
