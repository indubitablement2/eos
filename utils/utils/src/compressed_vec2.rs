use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Vec2 compressed to 4 bytes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CVec2<const R: u16> {
    pub x: u16,
    pub y: u16,
}
impl<const R: u16> CVec2<R> {
    pub const RANGE: f32 = R as f32;

    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn from_vec2(v: Vec2) -> Self {
        let v = (v / Self::RANGE + 0.5) * u16::MAX as f32;
        Self::new(v.x as u16, v.y as u16)
    }

    pub fn to_vec2(self) -> Vec2 {
        let v = Vec2::new(self.x as f32, self.y as f32);
        (v / u16::MAX as f32 - 0.5) * Self::RANGE
    }
}

#[test]
fn test_cvec2() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    const RANGE: u16 = 32767u16;
    let rangef = RANGE as f32;

    for i in 0..10000 {
        let vec2 = (rng.gen::<Vec2>() - 0.5) * rangef;
        let cvec2: CVec2<RANGE> = CVec2::from_vec2(vec2);
        let other = cvec2.to_vec2();

        assert!(
            vec2.abs_diff_eq(other, 0.5),
            "{}: {:?}, {:?}",
            i,
            vec2,
            other
        );
    }
}
