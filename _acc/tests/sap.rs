use acc::*;
// use acc::
use rand::prelude::*;

fn chungus() -> AABB {
    let mut rng = rand::thread_rng();

    let scale = rng.gen_range(0.0f32..32.0);
    let pos_x = rng.gen_range(-1024.0..1024.0);
    let pos_y = rng.gen_range(-1024.0..1024.0);

    AABB {
        left: pos_x - scale,
        top: pos_y - scale,
        right: pos_x + scale,
        bot: pos_y + scale,
    }
}

#[test]
fn test_empty_acc() {
    let acc: Sap<usize, AABB> = Sap::new();
    acc.intersect(&AABB::default(), |_, _| {
        assert!(false, "acc is empty");
        false
    });
}

#[test]
fn test_random_acc() {
    let mut rng = thread_rng();

    // Random test.
    for _ in 0..500 {
        let mut acc: Sap<usize, AABB> = Sap::new();
        for _ in 0..100 {
            let aabb = chungus();
            let mut expected_result = Vec::new();
    
            // Add aabbs.
            for i in 0..rng.gen_range(0..32) {
                let other = chungus();
                acc.queue(i, other);
    
                if aabb.intersect(&other) {
                    expected_result.push(i);
                }
            }
    
            expected_result.sort();
    
            acc.update();
    
            let mut result = Vec::new();
            acc.intersect(&aabb, |_, i| {
                result.push(*i);
                false
            });
            result.sort();
    
            assert_eq!(result, expected_result, "{:#?}\n{:#?}", aabb, &acc);
        }
    }
}

#[test]
fn test_random_acc_pair() {
    let mut rng = thread_rng();

    // Random test.
    for _ in 0..500 {
        let mut acc: Sap<usize, AABB> = Sap::new();
        for _ in 0..100 {
            let aabbs = (0..rng.gen_range(0usize..32))
                .map(|i| (i, rng.gen::<AABB>()))
                .collect::<Vec<_>>();
    
            acc.extend(aabbs.iter().copied());
            acc.update();
    
            let mut expected_result = Vec::new();
            for (i, &(id, aabb)) in aabbs.iter().enumerate() {
                for &(other_id, other_aabb) in aabbs[i..].iter().skip(1) {
                    if other_id != id && aabb.intersect(&other_aabb) {
                        expected_result.push((id, other_id));
                    }
                }
            }
            expected_result.sort();
    
            let mut result = Vec::new();
            acc.intersecting_pairs(None, |(_, &a), (_, &b)| {
                result.push((a.min(b), a.max(b)));
            });
            result.sort();
    
            assert_eq!(result, expected_result, "{:#?}", &acc);
        }
    }
}
