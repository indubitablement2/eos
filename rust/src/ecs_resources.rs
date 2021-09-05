pub struct GameParameter {
    pub drag: f32, // Velocity is multiplied by this each tick.
}

pub struct Time {
    pub tick: u32
}
pub struct Terrain {
    pub width: u16,
    pub height: u16,
    pub terrain: Vec<u8>,
}