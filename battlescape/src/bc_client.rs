use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BattlescapeClient {
    pub last_active: u64,
    pub last_inputs: PlayerInput,
    /// If the client is actively controlling a ship.
    pub control: Option<ShipId>,
}
impl BattlescapeClient {
    /// 5 secs
    const INACTIVE_DELAY: u64 = 20 * 5;

    pub fn active(&self, tick: u64) -> bool {
        tick.saturating_sub(self.last_active) < Self::INACTIVE_DELAY
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WishRot {
    /// Rotate left/right with a force (-1.0..1.0).
    Force(f32),
    /// Rotate to face that point. 
    Toward(na::Vector2<f32>),
}
impl Default for WishRot {
    fn default() -> Self {
        Self::Force(0.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PlayerInput {
    pub wish_rot: WishRot,
    /// Vector with a maximun magnitude of 1.
    pub wish_dir: na::Vector2<f32>,
    /// The global angle to aim to.
    pub wish_aim: f32,
    // TODO: Bitfield
    /// Toggle firing selected weapon group.
    pub fire_toggle: bool,
    /// If `wish_dir` is relative to the ship's rotation.
    pub wish_dir_relative: bool,
    /// Ignore wish_dir and try to cancel current velocity.
    pub stop: bool,
}
impl PlayerInput {
    pub fn validate(mut self) -> Self {
        // TODO: Remove inf/nan

        // TODO: Check that inputs are valid.
        self.wish_dir = self.wish_dir.cap_magnitude(1.0);

        self
    }
}
