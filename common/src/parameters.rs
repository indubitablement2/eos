/// Global Metascape parameters.
pub struct MetascapeParameters {
    /// The maximum distance to the center.
    pub bound: f32,
    /// Multiply fleet velocity every tick.
    pub movement_friction: f32,
}
impl Default for MetascapeParameters {
    fn default() -> Self {
        Self {
            bound: 1024.0,
            movement_friction: 0.95,
        }
    }
}
