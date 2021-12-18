pub struct TimeRes {
    pub tick: u32,
    pub cycle: u32,
}
impl Default for TimeRes {
    fn default() -> Self {
        Self { tick: 0, cycle: 0 }
    }
}
impl TimeRes {
    /// Increment tick by one returning if a new cycle started.
    pub fn increment(&mut self) -> bool {
        match self.tick.checked_add(1) {
            Some(new_tick) => {
                self.tick = new_tick;
                false
            }
            None => {
                self.tick = 0;
                self.cycle += 1;
                true
            }
        }
    }
}
