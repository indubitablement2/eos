use glam::Vec2;

/// Where the fleet wish to move.
#[derive(Debug, Clone, Copy)]
pub struct WishPosition {
    /// Where the fleet will try to move to.
    target: Option<Vec2>,
    /// Fleet want to reduce its movement speed.
    /// This is always in the range `0.0..=1.0`.
    movement_multiplier: f32,
}
impl WishPosition {
    /// Reset the wish position's target to none.
    pub fn stop(&mut self) {
        self.target = None;
        self.movement_multiplier = 1.0;
    }

    /// Set the target and movement multiplier.
    pub fn set_wish_position(&mut self, target: Vec2, movement_multiplier: f32) {
        self.target = Some(target);
        self.movement_multiplier = movement_multiplier.clamp(0.0, 1.0);
    }

    /// Get a reference to the movement multiplier.
    pub fn movement_multiplier(&self) -> f32 {
        self.movement_multiplier
    }

    /// Get a reference to the target.
    pub fn target(&self) -> Option<Vec2> {
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
