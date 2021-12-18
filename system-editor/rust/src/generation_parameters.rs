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
    /// Systems will not generate outside this radius.
    pub bound: f32,

    pub radius_min: f32,
    pub radius_max: f32,
    pub min_distance: f32,

    pub system_density: f32,
    pub system_size: f32,
}
impl Default for GenerationParameters {
    fn default() -> Self {
        Self {
            seed: 0,
            bound: 1024.0,
            radius_min: 32.0,
            radius_max: 128.0,
            min_distance: 16.0,
            system_density: 1.0,
            system_size: 1.0,
        }
    }
}
