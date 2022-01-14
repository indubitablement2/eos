pub struct Parameters {
    /// The maximum distance to the center.
    pub world_bound: f32,
    /// Multiply velocity every tick.
    pub friction: f32,
}
impl Default for Parameters {
    fn default() -> Self {
        Self {
            world_bound: u16::MAX as f32,
            friction: 0.9,
        }
    }
}
