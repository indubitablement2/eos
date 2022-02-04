use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Weither `wish_rot` is relative (apply a force left/right) or
    /// absolute (rotate to this this angle).
    pub wish_rot_absolute: bool,
    /// The direction (left/right) and force or
    /// the absolute rotation wanted to rotate the ship.
    pub wish_rot: u16,
    /// Angle and force wanted to translate the ship.
    pub wish_dir: (u16, u8),
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            wish_rot_absolute: false,
            wish_rot: u16::MAX / 2,
            wish_dir: (0, 0),
        }
    }
}
