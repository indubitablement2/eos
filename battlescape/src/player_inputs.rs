use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum WishRot {
    /// Rotate left/right.
    Relative(f32),
    /// Rotate to face a position in world space.
    FaceWorldPositon(f32, f32),
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct WishDir {
    pub angle: f32,
    pub force: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerInput {
    /// If `wish_rot.1` is nan determine if this is relative force
    /// or a world position.
    ///
    /// The direction (left/right) and force or
    /// the world position to face wanted to rotate the ship.
    wish_rot: (f32, f32),
    /// Angle and force wanted to translate the ship.
    wish_dir: (u16, u8),
    /// The world position to aim to.
    wish_aim: (f32, f32),
    /// Toggle firing selected weapon group.
    fire_toggle: bool,
}
impl PlayerInput {
    pub fn set_wish_rot(&mut self, wish_rot: WishRot) {
        match wish_rot {
            WishRot::Relative(r) => {
                self.wish_rot = (r, f32::NAN);
            }
            WishRot::FaceWorldPositon(x, y) => self.wish_rot = (x, y),
        }
    }

    /// Returned value is garanty to be valid (not nan or force above 1.0 / less than 0.0).
    pub fn get_wish_rot(&self) -> WishRot {
        if self.wish_rot.0.is_nan() {
            WishRot::Relative(0.0)
        } else if self.wish_rot.1.is_nan() {
            WishRot::Relative(self.wish_rot.0.clamp(-1.0, 1.0))
        } else {
            WishRot::FaceWorldPositon(self.wish_rot.0, self.wish_rot.1)
        }
    }

    pub fn set_wish_dir(&mut self, angle: f32, force: f32) {
        self.wish_dir.0 = ((angle / TAU) * u16::MAX as f32) as u16;
        self.wish_dir.1 = (force * u8::MAX as f32) as u8;
    }

    /// Return the angle `0.0..TAU` and force `0.0..1.0`.
    pub fn get_wish_dir(&self) -> WishDir {
        WishDir {
            angle: self.wish_dir.0 as f32 / u16::MAX as f32 * TAU,
            force: self.wish_dir.1 as f32 / u8::MAX as f32,
        }
    }

    /// Set the weapon aim position in world space.
    pub fn set_wish_aim(&mut self, x: f32, y: f32) {
        self.wish_aim = (x, y);
    }

    /// Get the weapon aim position in world space.
    pub fn get_wish_aim(&self) -> (f32, f32) {
        if self.wish_aim.0.is_nan() || self.wish_aim.1.is_nan() {
            (0.0, 0.0)
        } else {
            self.wish_aim
        }
    }

    /// Set the player input's fire toggle.
    pub fn set_fire_toggle(&mut self, fire_toggle: bool) {
        self.fire_toggle = fire_toggle;
    }

    /// Get the player input's fire toggle.
    pub fn get_fire_toggle(&self) -> bool {
        self.fire_toggle
    }
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            wish_rot: (0.0, f32::NAN),
            wish_dir: (0, 0),
            wish_aim: (0.0, 0.0),
            fire_toggle: false,
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
        i.set_wish_rot(a);
        assert!(i.get_wish_rot().eq(&a));
    }

    for _ in 0..1000 {
        let a =
            WishRot::FaceWorldPositon(rng.gen_range(-128.0..128.0), rng.gen_range(-128.0..128.0));
        i.set_wish_rot(a);
        assert!(i.get_wish_rot().eq(&a));
    }

    for _ in 0..1000 {
        let wish_dir = WishDir {
            angle: rng.gen_range(0.0..TAU),
            force: rng.gen_range(0.0..1.0),
        };
        i.set_wish_dir(wish_dir.angle, wish_dir.force);
        let r = i.get_wish_dir();
        assert!((r.angle - wish_dir.angle).abs() < 0.001);
        assert!((r.force - wish_dir.force).abs() < 0.01);
    }
}
