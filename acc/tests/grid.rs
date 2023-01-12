use acc::bounding_shape::*;
use acc::grid::*;
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
fn test_empty_grid() {
    let grid: Grid<AABB> = Default::default();
    grid.intersect(&AABB::default(), &mut Default::default(), |_, _| {
        assert!(false, "acc is empty");
        false
    });
}

#[test]
fn test_random_grid() {
    let mut rng = thread_rng();

    // Random test.
    for _ in 0..1000 {
        let mut grid: Grid<AABB> = Default::default();
        for _ in 0..50 {
            let aabb = chungus();
            let mut expected_result = Vec::new();

            // Add aabbs.
            for i in 0..rng.gen_range(0u32..16) {
                let other = chungus();
                let r = grid.queue(other);

                assert_eq!(r, i);

                if aabb.intersect(&other) {
                    expected_result.push(i);
                }
            }

            expected_result.sort();

            grid.update();

            let mut result = Vec::new();
            grid.intersect(&aabb, &mut Default::default(), |i, _| {
                result.push(i);
                false
            });
            result.sort();

            assert_eq!(result, expected_result, "{:#?}\n{:#?}", aabb, &grid);
            // println!("{:#?}", &grid);
        }
    }
}

#[test]
fn test_random_grid_pair() {
    let mut rng = thread_rng();

    for _ in 0..1000 {
        let mut grid: Grid<AABB> = Default::default();
        for _ in 0..50 {
            let aabbs = (0..rng.gen_range(0..32))
                .map(|_| chungus())
                .collect::<Vec<_>>();

            grid.extend(aabbs.iter().copied());
            grid.update();

            let mut expected_result = Vec::new();
            for (i, aabb) in aabbs.iter().enumerate() {
                let index = i as u32;
                for (other_aabb, other_index) in aabbs[i..].iter().zip(index..).skip(1) {
                    if other_index != index && aabb.intersect(other_aabb) {
                        expected_result.push((index, other_index));
                    }
                }
            }
            expected_result.sort();

            let mut result = Vec::new();
            grid.intersecting_pairs(|(a, a_shape), (b, b_shape)| {
                assert!(a_shape.intersect(b_shape));
                result.push((a.min(b), a.max(b)));
            });
            result.sort();

            assert_eq!(result, expected_result, "{:#?}", &grid);
        }
    }
}
