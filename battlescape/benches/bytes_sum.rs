#![feature(test)]
#![feature(slice_as_chunks)]

use rand::{thread_rng, Rng};
use test::{black_box, Bencher};
extern crate test;

fn v() -> Vec<u8> {
    let mut rng = thread_rng();
    (0..20000).into_iter().map(|_| rng.gen::<u8>()).collect()
}

#[bench]
fn sum_u128(b: &mut Bencher) {
    let v = v();

    b.iter(|| {
        let (c, r) = v.as_chunks();
        let r = c
            .into_iter()
            .fold(0u128, |acc, &x| acc.wrapping_add(u128::from_le_bytes(x)))
            .wrapping_add(r.into_iter().fold(0u128, |acc, &x| acc + x as u128));
        black_box(r);
    });
}

#[bench]
fn sum_u64(b: &mut Bencher) {
    let v = v();

    b.iter(|| {
        let (c, r) = v.as_chunks();
        let r = c
            .into_iter()
            .fold(0u64, |acc, &x| acc.wrapping_add(u64::from_le_bytes(x)))
            .wrapping_add(r.into_iter().fold(0u64, |acc, &x| acc + x as u64));
        black_box(r);
    });
}

#[bench]
fn sum_u32(b: &mut Bencher) {
    let v = v();

    b.iter(|| {
        let (c, r) = v.as_chunks();
        let r = c
            .into_iter()
            .fold(0u32, |acc, &x| acc.wrapping_add(u32::from_le_bytes(x)))
            .wrapping_add(r.into_iter().fold(0u32, |acc, &x| acc + x as u32));
        black_box(r);
    });
}
