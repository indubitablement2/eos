use glam::Vec2;

pub struct GenerationMask {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<f32>,
    /// Multiply the value form the buffer.
    pub multiplier: f32,
}
impl GenerationMask {
    /// Try to sample the buffer at the given position or return 1.0.
    pub fn sample(&self, uv: Vec2) -> f32 {
        let x = (uv.x * self.width as f32) as usize;
        let y = (uv.y * self.height as f32) as usize;
        *self.buffer.get(x + y * self.width).unwrap_or(&1.0) * self.multiplier
    }
}
impl Default for GenerationMask {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            buffer: vec![],
            multiplier: 1.0,
        }
    }
}

pub struct GenerationParameters {
    pub seed: u64,

    pub system_density: GenerationMask,
    pub system_size: GenerationMask,
}
impl Default for GenerationParameters {
    fn default() -> Self {
        Self {
            seed: 0,
            system_density: Default::default(),
            system_size: Default::default(),
        }
    }
}
