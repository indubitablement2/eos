use std::{time::{Duration, Instant}, thread::sleep};

/// Call the closure every interval until it return true.
/// 
/// Will call the closure every interval or as fast as the system can handle.
/// Will not death spiral.
/// 
/// ## Example:
/// interval is `100 ms`
/// - closure `80 ms`
/// - sleep `20 ms`
/// - closure `150 ms` (no sleep)
/// - closure `25 ms` 
/// - sleep `25 ms` (`100 ms` - (`50 ms` over from last update + `25 ms`))
pub fn interval(interval: Duration, mut closure: impl FnMut() -> bool) {
    let mut last_start = Instant::now();
    loop {
        let delta = last_start.elapsed();
        // Time alocated for this update.
        let update_duration = interval.saturating_sub(delta.saturating_sub(interval));
        last_start = Instant::now();

        // update
        if closure() {
            break;
        }

        // Sleep for the remaining time.
        if let Some(remaining) = update_duration.checked_sub(last_start.elapsed()) {
            sleep(remaining);
        }
    }
}