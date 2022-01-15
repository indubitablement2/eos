#[derive(Debug, Clone, Copy)]
pub struct Configs {
    pub system_draw_distance: f32,
}
impl Default for Configs {
    fn default() -> Self {
        Self {
            system_draw_distance: 256.0,
        }
    }
}
