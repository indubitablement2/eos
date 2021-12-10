pub struct TimeRes {
    pub tick: u64,
}
impl Default for TimeRes {
    fn default() -> Self {
        Self { tick: 0 }
    }
}
