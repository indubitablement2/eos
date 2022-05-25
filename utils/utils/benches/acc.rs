#![feature(test)]

use glam::Vec2;
use rand::prelude::*;
use std::time::Instant;
use test::{black_box, Bencher};
use utils::acc::*;

extern crate test;

const NUM: u32 = 30000;
const BOUND: f32 = 4096.0;
const NUM_FILTER: u32 = 4;

fn chungus() -> Vec<(Collider<u32>, u32)> {
    let mut rng = rand::thread_rng();

    (0..NUM)
        .map(|i| {
            (
                Collider::new(
                    rng.gen::<Vec2>() * BOUND * 2.0 - BOUND,
                    rng.gen_range(0.0f32..32.0),
                    rng.gen_range(0..NUM_FILTER),
                ),
                i,
            )
        })
        .collect()
}

/// Does NOT update the acc.
fn into_acc(chungus: &[(Collider<u32>, u32)]) -> AccelerationStructure<u32, u32> {
    let mut acc = AccelerationStructure::new();
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
#[bench]
fn bench_intersect_collider(b: &mut Bencher) {
    let to_test = chungus();
    let mut acc = into_acc(&to_test);

    let now = Instant::now();
    acc.update();
    println!("{}", now.elapsed().as_micros());

    b.iter(|| {
        for (collider, _) in to_test.iter() {
            acc.intersect_collider(*collider, |other| {
                black_box(&other);
                false
            });
        }
    });
}
