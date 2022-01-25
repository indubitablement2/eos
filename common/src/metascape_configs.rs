pub struct MetascapeConfigs {
    /// Multiply velocity every tick.
    pub friction: f32,
}
impl Default for MetascapeConfigs {
    fn default() -> Self {
        Self { friction: 0.95 }
    }
}
