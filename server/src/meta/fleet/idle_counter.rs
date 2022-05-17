/// How long this entity has been without velocity.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdleCounter {
    counter: u32,
}
impl IdleCounter {
    /// Delay before a fleet without velocity is considered idle in tick.
    const IDLE_DELAY: u32 = 60;

    pub fn increment(&mut self) {
        self.counter += 1;
    }

    pub fn reset(&mut self) {
        self.counter = 0;
    }

    pub fn is_idle(self) -> bool {
        self.counter >= Self::IDLE_DELAY
    }

    /// Will return true only when the fleet start idling.
    pub fn just_stated_idling(self) -> bool {
        self.counter == Self::IDLE_DELAY
    }
}
