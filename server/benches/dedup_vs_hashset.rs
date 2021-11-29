#![feature(test)]

use ahash::AHashSet;
use rand::Rng;
use test::{Bencher, black_box};

extern crate test;

/// How large is the array to test.
const NUM_TEST: usize = 32;
/// What is the range of possible value in the array.
const VALUE_RANGE: u32 = 16;

#[bench]
fn dedup_unstable_bench(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    b.iter(|| {
        let mut vec = Vec::with_capacity(NUM_TEST);

        for _ in 0..NUM_TEST {
            let v = rng.gen::<u32>() % VALUE_RANGE;
            vec.push(v);
        }

        vec.sort_unstable();
        vec.dedup();

        for v in vec.iter() {
            black_box(v);
        }
    });
}

#[bench]
fn dedup_stable_bench(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    b.iter(|| {
        let mut vec = Vec::with_capacity(NUM_TEST);

        for _ in 0..NUM_TEST {
            let v = rng.gen::<u32>() % VALUE_RANGE;
            vec.push(v);
        }

        vec.sort();
        vec.dedup();

        for v in vec.iter() {
            black_box(v);
        }
    });
}

#[bench]
fn hashset_bench(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    b.iter(|| {
        let mut hashset = AHashSet::with_capacity(NUM_TEST);

        for _ in 0..NUM_TEST {
            let v = rng.gen::<u32>() % VALUE_RANGE;
            hashset.insert(v);
        }

        for v in hashset.iter() {
            black_box(v);
        }
    });
}