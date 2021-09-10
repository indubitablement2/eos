pub struct GameParameterRes {
    pub drag: f32, // Velocity is multiplied by this each tick.
}

pub struct TimeRes {
    /// One tick is one second of simulated time.
    pub tick: u32,
    /// Time in second that is not a full tick. When above 1, tick is incremented and tick_accumulator is reduced by 1.
    pub time_accumulator: f32,
    /// Delta from Godot.
    pub delta: f32,
}
