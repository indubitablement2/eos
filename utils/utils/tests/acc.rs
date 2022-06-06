use glam::Vec2;
use rand::prelude::*;
use utils::acc::*;

#[test]
fn test_empty_acc() {
    let acc: AccelerationStructure<Circle, usize> = AccelerationStructure::new();
    acc.intersect_collider(&Circle::default(), |_, i| {
        assert!(*i > 0);
        false
    });
}

#[test]
fn test_random_colliders_acc() {
    let mut rng = thread_rng();

    // Random test.
    for _ in 0..50000 {
        let mut acc: AccelerationStructure<Circle, usize> = AccelerationStructure::new();

        let og_collider = Circle {
            center: rng.gen::<Vec2>() * 96.0 - 48.0,
            radius: rng.gen_range(0.0f32..16.0),
        };

        let mut expected_result = Vec::new();

        // Add colliders.
        for i in 0..rng.gen_range(0..4) {
            let new_collider = Circle {
                center: rng.gen::<Vec2>() * 96.0 - 48.0,
                radius: rng.gen_range(0.0f32..16.0),
            };
            acc.push(&new_collider, i);

            if og_collider.intersection_test(&new_collider) {
                expected_result.push(i);
            }
        }

        expected_result.sort();

        acc.update();

        let mut result = Vec::new();
        acc.intersect_collider(&og_collider, |_, i| {
            result.push(*i);
            false
        });
        result.sort();

        // if result != expected_result {
        //     panic!("Hello :)")
        // }
        assert_eq!(result, expected_result, "{:#?}\n{:#?}", og_collider, &acc);
    }
}
