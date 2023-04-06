use std::f32::consts::TAU;

use rand::prelude::*;
use rapier2d::na;
use rapier2d::prelude::*;

fn rand() -> (na::Vector2<f32>, na::UnitComplex<f32>, f32) {
    let angle = random::<f32>() * TAU;
    (
        na::vector![random::<f32>() - 0.5, random::<f32>() - 0.5] * 20.0,
        na::UnitComplex::new(angle),
        angle,
    )
}

fn main() {
    // Undo a rotation.
    for _ in 0..1000 {
        let (v, r, _) = rand();
        let tr = r.transform_vector(&v);

        let dif = r.conjugate().transform_vector(&tr) - v;

        assert!(dif.x.abs() < 0.001 && dif.y.abs() < 0.001, "{:?}", dif);
    }

    // Get the angle to a target vector.
    for i in 0..1000 {
        let (target, rot, _) = rand();

        // v1
        let target_complex = na::Complex::new(target.x, target.y);
        let target_rot = na::UnitComplex::from_complex(target_complex);

        let needed_rot = rot.rotation_to(&target_rot);
        let needed_angle = needed_rot.angle();

        let result_rot = na::UnitComplex::new(rot.angle() + needed_angle);
        let dif = target_rot.complex() - result_rot.complex();
        assert!(
            dif.re.abs() < 0.001 && dif.im.abs() < 0.001,
            "{}, {:?}",
            i,
            dif
        );
        drop((needed_rot, needed_angle, result_rot, dif));

        // v2
        let target_angle = na::RealField::atan2(target.y, target.x);
        let needed_angle = target_angle - rot.angle();

        let result_rot = na::UnitComplex::new(rot.angle() + needed_angle);
        let dif = target_rot.complex() - result_rot.complex();
        assert!(
            dif.re.abs() < 0.001 && dif.im.abs() < 0.001,
            "{}, {:?}",
            i,
            dif
        );
    }
}
