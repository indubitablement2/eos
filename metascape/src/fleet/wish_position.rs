use super::*;

/// Where the fleet wish to move.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WishPosition {
    /// Where the fleet will try to move to.
    pub target: Option<na::Vector2<f32>>,
    /// Fleet want to reduce its movement speed.
    /// This is always in the range `0.0..=1.0`.
    pub movement_multiplier: f32,
}
impl WishPosition {
    /// Reset the wish position's target to none.
    pub fn stop(&mut self) {
        self.target = None;
        self.movement_multiplier = 1.0;
    }

    /// Set the target and movement multiplier.
    pub fn set_wish_position(&mut self, target: na::Vector2<f32>, movement_multiplier: f32) {
        self.target = Some(target);
        self.movement_multiplier = movement_multiplier.clamp(0.0, 1.0);
    }

    /// Get a reference to the movement multiplier.
    pub fn movement_multiplier(&self) -> f32 {
        self.movement_multiplier
    }

    /// Get a reference to the target.
    pub fn target(&self) -> Option<na::Vector2<f32>> {
        self.target
    }
}
impl Default for WishPosition {
    fn default() -> Self {
        Self {
            target: None,
            movement_multiplier: 1.0,
        }
    }
}
