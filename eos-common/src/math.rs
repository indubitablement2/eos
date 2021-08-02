use glam::Vec2;

// pub fn vec2_cartesian_to_polar(location_from: u16, location_to: (u16, Vec2)) -> Vec2 {
//     todo!()
// }

/// Force that will lead to 0 velocity.
pub fn steering_break(delta: f32, vel: Vec2, accel: f32) -> Vec2 {
    -vel.clamp_length_max(accel * delta)
}

/// Try to push itself directly toward target as if pulled by gravity.
/// - No velocity/direction correction.
/// - Prone to rotating around target.
/// - Will overshoot.
pub fn steering_gravity(delta: f32, pos: Vec2, accel: f32, t_pos: Vec2) -> Vec2 {
    // A vector from pos toward t_pos.
    let desired_velocity = t_pos - pos;

    desired_velocity.clamp_length_max(accel * delta)
}

/// Go as fast as possible toward target.
/// - Correct its direction without losing speed.
// /// - Fastest direction correction at dir_correction == 0.
/// - Will overshoot.
pub fn steering_seek(delta: f32, pos: Vec2, vel: Vec2, accel: f32, t_pos: Vec2) -> Vec2 {
    // A vector from pos toward t_pos of lenght == vel.
    let desired_velocity_max = (t_pos - pos).clamp_length_max(vel.length() + accel * delta);

    (desired_velocity_max - vel).clamp_length_max(accel * delta)
}

/// Asume pos is the origin.
/// Go as fast as possible toward target.
/// - Correct its direction without losing speed.
// /// - Fastest direction correction at dir_correction == 0.
/// - Will overshoot.
pub fn steering_local_seek(delta: f32, vel: Vec2, speed: f32, accel: f32, target: Vec2) -> Vec2 {
    (target.clamp_length_max(speed + accel * delta) - vel).clamp_length_max(accel * delta)
}

// /// Go as fast as possible toward target.
// /// - Correct its direction and velocity.
// /// - Will break when on target and return true.
// /// - It is the same as steering_seek when far away from target.
// /// - Become steering_break when under arrival_threshold_squared or ETA <= time to zero vel.
// /// - A good arrival_threshold_squared could be a fleet radius.
// pub fn steering_arrival(delta: f32, pos: Vec2, vel: Vec2, accel: f32, arrival_threshold_squared: f32, t_pos: Vec2) -> (Vec2, bool) {
//     // A vector from pos toward t_pos of lenght == vel.
//     let desired_velocity = t_pos - pos;

//     // Break if we are close enough to target.
//     if desired_velocity.length_squared() <= arrival_threshold_squared {
//         return (steering_break(delta, vel, accel), true);
//     }
//     // Break if ETA <= time to zero vel.

//     ((desired_velocity - vel).clamp_length_max(accel * delta), false)
// }

/// How many seconds to reach max speed.
pub fn time_to_max_speed(current_speed: f32, accel: f32, max_speed: f32) -> f32 {
    (max_speed - current_speed) / accel
}

/// How many seconds to 0 speed at current speed and accel.
pub fn time_to_zero_vel(current_speed: f32, accel: f32) -> f32 {
    current_speed / accel
}

/// Approximate how many seconds to target.
/// - Asume velocity does not change.
pub fn eta_current(pos: Vec2, vel: Vec2, t_pos: Vec2) -> f32 {
    let desired_velocity = t_pos - pos;
    let useful_vel = useful_velocity(vel, desired_velocity.normalize());

    desired_velocity.length() / useful_vel.length()
}

// /// Approximate how many seconds to target.
// /// - Asume velocity change to seek target.
// pub fn eta_seek(pos: Vec2, vel: Vec2, accel: f32, t_pos: Vec2) -> f32 {
//
// }

/// Useful velocity toward normalized direction.
pub fn useful_velocity(vel: Vec2, wish_dir: Vec2) -> Vec2 {
    vel.project_onto_normalized(wish_dir) * wish_dir
}
