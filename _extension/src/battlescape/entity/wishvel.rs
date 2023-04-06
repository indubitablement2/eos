use super::*;

/// Increate acceleration by this much when stopping.
const STOP_ACC_MULTIPLIER: f32 = 1.05;

pub fn angvel_stop(angvel: f32, angacc: f32) -> f32 {
    if ComplexField::abs(angvel) < 0.001 {
        0.0
    } else {
        angvel + RealField::clamp(-angvel, -angacc, angacc) * STOP_ACC_MULTIPLIER
    }
}

pub fn angvel_force(angvel: f32, force: f32, angacc: f32, max_angvel: f32) -> f32 {
    if angvel > max_angvel {
        if ComplexField::signum(force) == ComplexField::signum(angvel) {
            // Trying to go in the same dir as current velocity while speed is over max.
            // Ignore force, slow down to max speed instead.
            RealField::max(angvel - angacc, max_angvel)
        } else {
            // Trying to go in the opposite dir as current velocity while speed is over max.
            let maybe = angvel + force * angacc;
            if maybe > max_angvel {
                // Ignore force, slow down as much as possible to reach max speed instead.
                RealField::max(angvel - angacc, max_angvel)
            } else {
                // Force is enough to slow down to max speed.
                RealField::max(maybe, -max_angvel)
            }
        }
    } else if angvel < -max_angvel {
        if ComplexField::signum(force) == ComplexField::signum(angvel) {
            // Trying to go in the same dir as current velocity while speed is over max.
            // Ignore force, slow down to max speed instead.
            RealField::min(angvel + angacc, -max_angvel)
        } else {
            // Trying to go in the opposite dir as current velocity while speed is over max.
            let maybe = angvel + force * angacc;
            if maybe > max_angvel {
                // Ignore force, slow down as much as possible to reach max speed instead.
                RealField::min(angvel + angacc, -max_angvel)
            } else {
                // Force is enough to slow down to max speed.
                RealField::min(maybe, max_angvel)
            }
        }
    } else {
        // Speed is under max.
        RealField::clamp(angvel + force * angacc, -max_angvel, max_angvel)
    }
}

pub fn angvel_target(angvel: f32, wish_rot_offset: f32, angacc: f32, max_angvel: f32) -> f32 {
    if ComplexField::abs(wish_rot_offset) < 0.005 {
        angvel_stop(angvel, angacc)
    } else if ComplexField::signum(wish_rot_offset) == ComplexField::signum(angvel) {
        // Calculate the time to reach 0 angvel.
        let time_to_stop = ComplexField::abs(angvel * DT) / (angacc);

        // Calculate the time to reach the target.
        let time_to_target = ComplexField::abs(wish_rot_offset / angvel);

        if time_to_target < time_to_stop {
            // We will overshot the target, so we need to slow down.
            angvel_stop(angvel, angacc)
        } else {
            angvel_force(angvel, wish_rot_offset, angacc, max_angvel)
        }
    } else {
        // We are going in the opposite direction, so we can go at full speed.
        angvel_force(angvel, wish_rot_offset, angacc, max_angvel)
    }
}

pub fn linvel_wish_linvel(
    linvel: na::Vector2<f32>,
    wish_vel: na::Vector2<f32>,
    linacc: f32,
) -> na::Vector2<f32> {
    let vel_change = (wish_vel - linvel).cap_magnitude(linacc);
    linvel + vel_change
}

pub fn linvel_stop(linvel: na::Vector2<f32>, linacc: f32) -> na::Vector2<f32> {
    if linvel.magnitude_squared() < 0.001 {
        na::zero()
    } else {
        linvel - linvel.cap_magnitude(linacc * STOP_ACC_MULTIPLIER)
    }
}
