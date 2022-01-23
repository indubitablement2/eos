pub struct Parameters {
    /// Multiply velocity every tick.
    pub friction: f32,
}
impl Default for Parameters {
    fn default() -> Self {
        Self { friction: 0.95 }
    }
}
