// use eos_common::location::Location;
use glam::*;

fn main() {
    let vel = vec2(10.0, 0.0);
    let wish_dir = vec2(10.0, -10.0).normalize();

    let result = vel.project_onto_normalized(wish_dir) * wish_dir;

    println!("Useful vel: {}. Useful speed: {}.", result, result.length());

    println!("{}", eos_common::math::eta_current(vec2(55.0, 123.0), vel, vec2(-22.0, 13.0)));

    let mut pos = vec2(-12345.123, 120.22);
    let mut vel = Vec2::ZERO;
    let accel = 2.0;
    let max_speed = 10.0;

    let t_pos = vec2(54326.85, -663.2);

    for i in 0..100 {
        let new_force = eos_common::math::steering_seek(0.1, pos, vel, accel, t_pos);

        let new_vel = (vel + new_force).clamp_length_max(max_speed);

        // if i % 10 == 0 {
        //     println!("Pos: {}, Vel: {}, Speed: {}", pos, vel, vel.length());
        // }

        if new_vel.abs_diff_eq(vel, 0.1) && vel.length_squared() >= max_speed * max_speed - 0.1 {
            print!("Cruising. ");
        }

        if new_vel.abs_diff_eq(vel, 0.2) {
            println!("T: {}, Vel: {}, Speed: {}", i as f32 * 0.1, vel, vel.length());
        }

        vel = new_vel;
        pos += vel;
    }
}
