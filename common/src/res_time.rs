pub struct TimeRes {
    pub tick: u32,
}
impl Default for TimeRes {
    fn default() -> Self {
        Self { tick: 0 }
    }
}
