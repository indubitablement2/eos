use std::f32::consts::TAU;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum WishRot {
    /// Rotate left/right.
    Relative(f32),
    /// Rotate to face a position in world space.
    FaceWorldPositon(f32, f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerInput {
    /// If `wish_rot.1` is nan detecmine if this is relative relative force
    /// or a world position.
    /// 
    /// The direction (left/right) and force or
    /// the world position to face wanted to rotate the ship.
    wish_rot: (f32, f32),
    /// Angle and force wanted to translate the ship.
    wish_dir: (u16, u8),
}

impl PlayerInput {
    /// Returned value is garanty to be valid (not nan or force above 1.0).
    pub fn uncompressed_wish_rot(&self) -> WishRot {
        if self.wish_rot.0.is_nan() {
            WishRot::Relative(0.0)
        } else if self.wish_rot.1.is_nan() {
            WishRot::Relative(self.wish_rot.0.clamp(-1.0, 1.0))
        } else {
            WishRot::FaceWorldPositon(self.wish_rot.0, self.wish_rot.1)
        }
    }

    pub fn compress_wish_rot(&mut self, wish_rot: WishRot) {
        match wish_rot {
            WishRot::Relative(r) => {
                self.wish_rot = (r, f32::NAN);
            }
            WishRot::FaceWorldPositon(x, y) => {
                self.wish_rot = (x, y)
            }
        }
    }

    /// Return the angle `0.0..TAU` and force `0.0..1.0`.
    pub fn uncompress_wish_dir(&self) -> (f32, f32) {
        (
            self.wish_dir.0 as f32 / u16::MAX as f32 * TAU,
            self.wish_dir.1 as f32 / u8::MAX as f32
        )
    }

    pub fn compress_wish_dir(&mut self, angle: f32, force: f32) {
        self.wish_dir.0 = ((angle / TAU) * u16::MAX as f32) as u16;
        self.wish_dir.1 = (force * u8::MAX as f32) as u8;
    }
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            wish_rot: (0.0, f32::NAN),
            wish_dir: (0, 0),
        }
    }
}

#[test]
fn test_player_input() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut i = PlayerInput::default();

    assert_ne!(WishRot::Relative(1.0), WishRot::Relative(1.1));

    for _ in 0..1000 {
        let a = WishRot::Relative(rng.gen_range(-1.0..1.0));
        i.compress_wish_rot(a);
        assert!(i.uncompressed_wish_rot().eq(&a));
    }

    for _ in 0..1000 {
        let a = WishRot::FaceWorldPositon(rng.gen_range(-128.0..128.0), rng.gen_range(-128.0..128.0));
        i.compress_wish_rot(a);
        assert!(i.uncompressed_wish_rot().eq(&a));
    }

    for _ in 0..1000 {
        let wish_dir = (rng.gen_range(0.0..TAU), rng.gen_range(0.0..1.0));
        i.compress_wish_dir(wish_dir.0, wish_dir.1);
        let r = i.uncompress_wish_dir();
        assert!((r.0 - wish_dir.0).abs() < 0.001);
        assert!((r.1 - wish_dir.1).abs() < 0.01);
    }
}