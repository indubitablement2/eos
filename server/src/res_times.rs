pub struct TimeRes {
    pub tick: u64,
}
impl TimeRes {
    pub fn new() -> Self {
        Self { tick: 0 }
    }
}
