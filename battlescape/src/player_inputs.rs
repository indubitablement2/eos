use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WishRot {
    /// Rotate left/right with a force (-1.0..1.0).
    Relative(f32),
    /// Rotate to this angle.
    Rotation(na::UnitComplex<f32>),
}
impl WishRot {
    pub fn new_relative(force: f32) -> Self {
        Self::Relative(force.clamp(-1.0, 1.0))
    }

    pub fn new_rotation(angle: f32) -> Self {
        Self::Rotation(na::UnitComplex::new(angle))
    }
}
impl Default for WishRot {
    fn default() -> Self {
        Self::Relative(0.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PlayerInput {
    pub wish_rot: WishRot,
    /// Vector with a maximun magnitude of 1.
    wish_dir: na::Vector2<f32>,
    /// The global angle to aim to.
    pub wish_aim: na::UnitComplex<f32>,
    /// Toggle firing selected weapon group.
    pub fire_toggle: bool,
}
impl PlayerInput {
    pub fn set_wish_dir(&mut self, wish_dir: na::Vector2<f32>) {
        self.wish_dir = wish_dir.cap_magnitude(1.0)
    }

    pub fn wish_dir(&self) -> na::Vector2<f32> {
        self.wish_dir
    }
}
