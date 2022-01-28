use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CVec2 {
    pub x: u16,
    pub y: u16,
}
impl CVec2 {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn from_vec2(v: Vec2, range: f32) -> Self {
        let v = (v / range + 0.5) * u16::MAX as f32;
        Self::new(v.x as u16, v.y as u16)
    }

    pub fn to_vec2(self, range: f32) -> Vec2 {
        let v = Vec2::new(self.x as f32, self.y as f32);
        (v / u16::MAX as f32 - 0.5) * range
    }
}

#[test]
fn test_cvec2() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let range = u16::MAX as f32 * 0.5;

    for i in 0..10000 {
        let vec2 = (rng.gen::<Vec2>() - 0.5) * range;
        let cvec2 = CVec2::from_vec2(vec2, range);
        let other = cvec2.to_vec2(range);

        assert!(vec2.abs_diff_eq(other, 0.5), "{}: {:?}, {:?}", i, vec2, other);
    }
}
