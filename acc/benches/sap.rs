#![feature(test)]

use acc::*;
use rand::prelude::*;
use std::{ops::Range, time::Instant};
use test::{black_box, Bencher};

extern crate test;

const PRE_SORT: bool = false;
const NUM: u32 = 30000;
const BOUND: f32 = 4096.0;
const RADIUS: Range<f32> = 0.0f32..16.0;

fn chungus() -> Vec<(u32, AABB)> {
    let mut rng = rand::thread_rng();

    let mut r: Vec<(u32, AABB)> = (0..NUM)
        .map(|i| {
            let scale = rng.gen_range(RADIUS);
            let pos_x = rng.gen_range(-BOUND..BOUND);
            let pos_y = rng.gen_range(-BOUND..BOUND);
            (
                i,
                AABB {
                    left: pos_x - scale,
                    top: pos_y - scale,
                    right: pos_x + scale,
                    bot: pos_y + scale,
                },
            )
        })
        .collect();

    if PRE_SORT {
        r.sort_by(|a, b| a.1.bot.partial_cmp(&b.1.bot).unwrap());
    }

    r
}

/// Does NOT update the acc.
fn into_acc(chungus: &[(u32, AABB)]) -> Sap<u32, AABB> {
    let mut acc = Sap::new();
    acc.extend(chungus.iter().copied());
    acc
}

/// Previous benches with a 2Ghz laptop:
/// - v0.0.2 20 ms (not filtered)
/// - v0.0.3 16 ms
/// - v0.0.4 28 ms (try IndexMap)
/// - v0.0.4 20 ms (use better threshold, remove IndexMap)
/// - v0.0.4 19 ms (improve threshold again)
/// - v0.0.4 23 ms (IndexMap return)
/// - v0.0.4 16 + 4 ms (stateless)
/// - v0.0.4 20 + 4 ms (no copy to SAPRow)
/// - v0.0.4 16 + 4 ms (copy to SAPRow)
/// - v0.0.5 10 + 6 ms (use aabb, no filter, draft!)
/// - v0.0.5 10 + 6 ms (3ms sort + 3ms copy) (pre-alocate)
/// - v0.0.6 8 + 6 ms (3ms sort + 3ms copy) (pure aabb)
/// - v0.0.7 8 + 6 ms (3ms sort + 3ms copy) (generic bounding shape)
#[bench]
fn bench_intersect(b: &mut Bencher) {
    let to_test = chungus();
    let mut acc = into_acc(&to_test);

    let now = Instant::now();
    acc.update();
    println!("{}", now.elapsed().as_micros());

    b.iter(|| {
        for (_, aabb) in to_test.iter() {
            acc.intersect(aabb, |_, i| {
                black_box(i);
                false
            });
        }
    });
}

/// - v0.0.6 6 + 6 ms (3ms sort + 3ms copy)
#[bench]
fn bench_intersect_pairs(b: &mut Bencher) {
    let to_test = chungus();
    let mut acc = into_acc(&to_test);

    let now = Instant::now();
    acc.update();
    println!("{}", now.elapsed().as_micros());

    b.iter(|| {
        acc.intersecting_pairs(None, |a, b| {
            black_box((a, b));
        });
    });
}

/// - v0.0.7 6 + 6 ms (3ms sort + 3ms copy)
#[bench]
fn bench_intersect_pairs_circle(b: &mut Bencher) {
    let to_test = chungus();
    let mut acc = Sap::new();
    acc.extend(to_test.iter().map(|(id, aabb)| {
        (
            *id,
            CircleBoundingShape {
                x: (aabb.right + aabb.left) * 0.5,
                y: (aabb.bot + aabb.top) * 0.5,
                r: aabb.width() * 0.5,
            },
        )
    }));

    let now = Instant::now();
    acc.update();
    println!("{}", now.elapsed().as_micros());

    b.iter(|| {
        acc.intersecting_pairs(None, |a, b| {
            black_box((a, b));
        });
    });
}
