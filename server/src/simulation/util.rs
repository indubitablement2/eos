use super::*;

pub fn integrate_linear_velocity(
    linear_velocity: Vec2,
    wish_linear_velocity: Vec2,
    linear_acceleration: f32,
    delta: f32,
) -> Vec2 {
    return linear_velocity
        + (wish_linear_velocity - linear_velocity).clamp_length_max(linear_acceleration * delta);
}

pub fn integrate_angular_velocity(
    angular_velocity: f32,
    wish_angular_velocity: f32,
    angular_acceleration: f32,
    delta: f32,
) -> f32 {
    return angular_velocity
        + f32::clamp(
            wish_angular_velocity - angular_velocity,
            -angular_acceleration * delta,
            angular_acceleration * delta,
        );
}

/// Return an angle such that `angle + this` point toward `to`.
/// Result is in the range `[-PI, PI]`.
pub fn angle_to(angle: f32, to: Vec2) -> f32 {
    return Vec2::from_angle(angle).angle_between(to);
}

#[test]
fn test_angle_to() {
    let epsilon = 0.001;

    approx::assert_relative_eq!(angle_to(0.0, vec2(1.0, 0.0)), 0.0, epsilon = epsilon);
    approx::assert_relative_eq!(
        angle_to(0.0, vec2(0.0, -1.0)),
        -FRAC_PI_2,
        epsilon = epsilon
    );
    approx::assert_relative_eq!(angle_to(0.0, vec2(0.0, 1.0)), FRAC_PI_2, epsilon = epsilon);

    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let a = rng.gen_range(-PI..PI);
        let b = a + rng.gen_range(-PI..PI);
        let expected = b - a;
        let v = Vec2::from_angle(b);

        approx::assert_relative_eq!(angle_to(a, v), expected, epsilon = epsilon);
    }
}
