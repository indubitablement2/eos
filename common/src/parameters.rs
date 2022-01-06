/// Global Metascape parameters.
pub struct MetascapeParameters {
    /// The maximum distance to the center.
    pub bound: f32,
    /// Multiply velocity every tick.
    pub friction: f32,
}
impl Default for MetascapeParameters {
    fn default() -> Self {
        Self {
            bound: 1024.0,
            friction: 0.95,
        }
    }
}
