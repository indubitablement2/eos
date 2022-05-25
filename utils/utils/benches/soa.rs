#![feature(test)]

use test::{black_box, Bencher};
use utils::*;

extern crate test;

const NUM: usize = 50000;

// 22/05/2022
// test soa_manual_math_50k                 ... bench:     273,823 ns/iter (+/- 19,144)
// test soa_query_math_50k                  ... bench:     379,182 ns/iter (+/- 57,784)
// test vec_math_50k                        ... bench:     686,858 ns/iter (+/- 148,400)
// test soa_manual_random_access_double_50k ... bench:     227,240 ns/iter (+/- 21,793)
// test soa_query_random_access_double_50k  ... bench:     226,085 ns/iter (+/- 8,452)
// test vec_random_access_double_50k        ... bench:     243,282 ns/iter (+/- 12,450)

#[derive(Debug, Clone, Fields, Columns, Components)]
struct C {
    x_pos: f32,
    y_pos: f32,
    x_vel: f32,
    y_vel: f32,
    acceleration: f32,
    x_wish_pos: f32,
    y_wish_pos: f32,
    name_string: String,
    surname_string: String,
    id: usize,
}

fn chungus(num: usize) -> Vec<C> {
    (0..num)
        .map(|i| {
            let f = i as f32;
            C {
                x_pos: f,
                y_pos: -f,
                x_vel: 0.0,
                y_vel: 0.0,
                acceleration: f,
                x_wish_pos: -f,
                y_wish_pos: f,
                name_string: format!("#{}", i),
                surname_string: format!("#{}", i),
                id: i,
            }
        })
        .collect()
}

#[bench]
fn soa_manual_math_50k(b: &mut Bencher) {
    let mut v = Soa::with_capacity(NUM);
    v.extend(chungus(NUM).into_iter());

    b.iter(|| {
        let (
            x_pos_ptr,
            y_pos_ptr,
            x_wish_pos_ptr,
            y_wish_pos_ptr,
            acceleration_ptr,
            x_vel_ptr,
            y_vel_ptr,
        ) = query_ptr!(
            v,
            C::x_pos,
            C::y_pos,
            C::x_wish_pos,
            C::y_wish_pos,
            C::acceleration,
            C::x_vel,
            C::y_vel
        );
        for i in 0..v.len() {
            let (x_pos, y_pos, x_wish_pos, y_wish_pos, acceleration, x_vel, y_vel) = unsafe {
                (
                    &*x_pos_ptr.add(i),
                    &*y_pos_ptr.add(i),
                    &*x_wish_pos_ptr.add(i),
                    &*y_wish_pos_ptr.add(i),
                    &*acceleration_ptr.add(i),
                    &mut *x_vel_ptr.add(i),
                    &mut *y_vel_ptr.add(i),
                )
            };

            let x_wish_dir = *x_pos - *x_wish_pos;
            let y_wish_dir = *y_pos - *y_wish_pos;
            let len_wish_dir = (x_wish_dir.powi(2) + y_wish_dir.powi(2)).sqrt();

            *x_vel += (x_wish_dir / len_wish_dir) * *acceleration;
            *y_vel += (y_wish_dir / len_wish_dir) * *acceleration;

            *x_vel *= 0.9;
            *y_vel *= 0.9;
        }

        let (x_pos_ptr, y_pos_ptr, x_vel_ptr, y_vel_ptr) =
            query_ptr!(v, C::x_pos, C::y_pos, C::x_vel, C::y_vel);
        for i in 0..v.len() {
            let (x_pos, y_pos, x_vel, y_vel) = unsafe {
                (
                    &mut *x_pos_ptr.add(i),
                    &mut *y_pos_ptr.add(i),
                    &*x_vel_ptr.add(i),
                    &*y_vel_ptr.add(i),
                )
            };

            *x_pos += *x_vel;
            *y_pos += *y_vel;
        }

        black_box(&mut v);
    });
}

#[bench]
fn soa_query_math_50k(b: &mut Bencher) {
    let mut v = Soa::with_capacity(NUM);
    v.extend(chungus(NUM).into_iter());

    b.iter(|| {
        for i in 0..v.len() {
            let (
                x_vel,
                y_vel,
                x_pos,
                y_pos,
                acceleration,
                x_wish_pos,
                y_wish_pos
            ) = query!(v, i, mut C::x_vel, mut C::y_vel, C::x_pos, C::y_pos, C::acceleration, C::x_wish_pos, C::y_wish_pos);

            let x_wish_dir = *x_pos - *x_wish_pos;
            let y_wish_dir = *y_pos - *y_wish_pos;
            let len_wish_dir = (x_wish_dir.powi(2) + y_wish_dir.powi(2)).sqrt();

            *x_vel += (x_wish_dir / len_wish_dir) * acceleration;
            *y_vel += (y_wish_dir / len_wish_dir) * acceleration;

            *x_vel *= 0.9;
            *y_vel *= 0.9;
        }

        for i in 0..v.len() {
            let (
                x_pos,
                y_pos,
                x_vel,
                y_vel,
            ) = query!(v, i, mut C::x_pos, mut C::y_pos, C::x_vel, C::y_vel);

            *x_pos += *x_vel;
            *y_pos += *y_vel;
        }

        black_box(&mut v);
    });
}

#[bench]
fn vec_math_50k(b: &mut Bencher) {
    let mut v = chungus(NUM);

    b.iter(|| {
        for s in v.iter_mut() {
            let x_wish_dir = s.x_pos - s.x_wish_pos;
            let y_wish_dir = s.y_pos - s.y_wish_pos;
            let len_wish_dir = (x_wish_dir.powi(2) + y_wish_dir.powi(2)).sqrt();

            s.x_vel += (x_wish_dir / len_wish_dir) * s.acceleration;
            s.y_vel += (y_wish_dir / len_wish_dir) * s.acceleration;

            s.x_vel *= 0.9;
            s.y_vel *= 0.9;
        }

        for s in v.iter_mut() {
            s.x_pos += s.x_vel;
            s.y_pos += s.y_vel;
        }

        black_box(&mut v);
    });
}

#[bench]
fn soa_manual_random_access_double_50k(b: &mut Bencher) {
    let mut v = Soa::with_capacity(NUM);
    v.extend(chungus(NUM).into_iter());

    b.iter(|| {
        let (name_string_ptr, surname_string_ptr) =
            query_ptr!(v, C::name_string, C::surname_string);

        for i in 0..v.len() {
            let i = (i >> 3) % v.len();
            let mut s = unsafe {
                (
                    &mut *name_string_ptr.add(i),
                    &mut *surname_string_ptr.add(i),
                )
            };
            black_box(&mut s);
        }
    });
}

#[bench]
fn soa_query_random_access_double_50k(b: &mut Bencher) {
    let mut v = Soa::with_capacity(NUM);
    v.extend(chungus(NUM).into_iter());

    b.iter(|| {
        for i in 0..v.len() {
            let i = (i >> 3) % v.len();
            let mut s = query!(v, i, mut C::name_string, mut C::surname_string);
            black_box(&mut s);
        }
    });
}

#[bench]
fn vec_random_access_double_50k(b: &mut Bencher) {
    let mut v = chungus(NUM);

    b.iter(|| {
        for i in 0..v.len() {
            let i = (i >> 3) % v.len();
            let values = &mut v[i];
            let mut s = (&mut values.name_string, &mut values.surname_string);
            black_box(&mut s);
        }
    });
}
