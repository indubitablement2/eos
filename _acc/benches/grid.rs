#![feature(test)]

use acc::bounding_shape::*;
use acc::grid::*;
use ahash::AHashSet;
use rand::prelude::*;
use std::{ops::Range, time::Instant};
use test::{black_box, Bencher};

extern crate test;

const NUM: u32 = 30000;
const BOUND: f32 = 4096.0;
const RADIUS: Range<f32> = 0.0f32..16.0;

fn chungus() -> Vec<AABB> {
    let mut rng = rand::thread_rng();

    (0..NUM)
        .map(|_| {
            let scale = rng.gen_range(RADIUS);
            let pos_x = rng.gen_range(-BOUND..BOUND);
            let pos_y = rng.gen_range(-BOUND..BOUND);

            AABB {
                left: pos_x - scale,
                top: pos_y - scale,
                right: pos_x + scale,
                bot: pos_y + scale,
            }
        })
        .collect::<Vec<_>>()
}

/// Does NOT update the grid.
fn into_grid(chungus: &[AABB]) -> Grid<AABB> {
    let mut grid: Grid<AABB> = Default::default();
    grid.extend(chungus.iter().copied());
    grid
}

/// Previous benches with a 2Ghz laptop:
/// - v0.0.8 8 + 3 update ms (grid)
#[bench]
fn bench_intersect_grid(b: &mut Bencher) {
    let to_test = chungus();
    let mut grid = into_grid(&to_test);

    let now = Instant::now();
    grid.update();
    println!("{}", now.elapsed().as_micros());

    let mut seen = AHashSet::new();

    b.iter(|| {
        for aabb in to_test.iter() {
            grid.intersect(aabb, &mut seen, |i, _| {
                black_box(i);
                false
            });
            seen.clear();
        }
    });

    println!("{}", grid.width() * grid.height());
}

/// - v0.0.6 6 + 6 ms (3ms sort + 3ms copy)
/// - v0.0.8 8 + 3 ms (grid)
#[bench]
fn bench_intersect_pairs_grid(b: &mut Bencher) {
    let to_test = chungus();
    let mut grid = into_grid(&to_test);

    let now = Instant::now();
    grid.update();
    println!("{}", now.elapsed().as_micros());

    b.iter(|| {
        grid.intersecting_pairs(|a, b| {
            black_box((a, b));
        });
    });
}

/// - v0.0.7 6 + 6 ms (3ms sort + 3ms copy)
/// - v0.0.8 6 + 4 ms (grid)
#[bench]
fn bench_intersect_pairs_grid_circle(b: &mut Bencher) {
    let to_test = chungus();
    let mut grid = Grid::default();
    grid.extend(to_test.iter().map(|aabb| CircleBoundingShape {
        x: (aabb.right + aabb.left) * 0.5,
        y: (aabb.bot + aabb.top) * 0.5,
        r: aabb.width() * 0.5,
    }));

    let now = Instant::now();
    grid.update();
    println!("{}", now.elapsed().as_micros());

    b.iter(|| {
        grid.intersecting_pairs(|a, b| {
            black_box((a, b));
        });
    });
}
