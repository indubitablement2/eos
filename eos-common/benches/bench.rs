#![feature(test)]
extern crate test;

use std::{
    convert::TryInto,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

/// Only use this if you are 100% sure your slice has 4 elements.
fn slice_to_array4(slice: &[u8]) -> [u8; 4] {
    slice.try_into().expect("slice with incorrect length")
}

#[bench]
fn bench_slice_to_array4(b: &mut test::Bencher) {
    let num_iter = 1000000u32;

    // Alloc bytes vec.
    let mut v = vec![0; (num_iter * 4).try_into().unwrap()];
    // Copy bytes into vec.
    for i in 0..num_iter {
        let start = (i * 4) as usize;
        let end = start + 4;
        v[start..end].copy_from_slice(&i.to_be_bytes());
    }

    let mut result = Vec::with_capacity(num_iter.try_into().unwrap());

    let now = std::time::Instant::now();

    b.iter(|| {
        for i in 0..num_iter {
            let start = (i * 4) as usize;
            let end = start + 4;

            // * What I actualy want to bench.
            let array4 = slice_to_array4(&v[start..end]);

            let value = u32::from_be_bytes(array4);

            result.push(value)
        }
    });

    println!("{:?}", now.elapsed());

    for i in 0..num_iter {
        assert_eq!(i, result[i as usize]);
    }
}

fn safer_slice_to_array4(slice: &[u8]) -> [u8; 4] {
    // Init the array.
    let mut arr: [u8; 4] = [0; 4];

    // Copy slice into array.
    arr.copy_from_slice(slice);

    arr
}

#[bench]
fn bench_safer_slice_to_array4(b: &mut test::Bencher) {
    let num_iter = 1000000u32;

    // Alloc bytes vec.
    let mut v = vec![0; (num_iter * 4).try_into().unwrap()];
    // Copy bytes into vec.
    for i in 0..num_iter {
        let start = (i * 4) as usize;
        let end = start + 4;
        v[start..end].copy_from_slice(&i.to_be_bytes());
    }

    let mut result = Vec::with_capacity(num_iter.try_into().unwrap());

    let now = std::time::Instant::now();

    b.iter(|| {
        for i in 0..num_iter {
            let start = (i * 4) as usize;
            let end = start + 4;

            // * What I actualy want to bench.
            let array4 = safer_slice_to_array4(&v[start..end]);

            let value = u32::from_be_bytes(array4);

            result.push(value)
        }
    });

    println!("{:?}", now.elapsed());

    for i in 0..num_iter {
        assert_eq!(i, result[i as usize]);
    }
}

fn safe_slice_to_array4(slice: &[u8]) -> [u8; 4] {
    if slice.len() != 4 {
        return [0u8; 4];
    }
    slice.try_into().unwrap()
}

#[bench]
fn bench_safe_slice_to_array4(b: &mut test::Bencher) {
    let num_iter = 1000000u32;

    // Alloc bytes vec.
    let mut v = vec![0; (num_iter * 4).try_into().unwrap()];
    // Copy bytes into vec.
    for i in 0..num_iter {
        let start = (i * 4) as usize;
        let end = start + 4;
        v[start..end].copy_from_slice(&i.to_be_bytes());
    }

    let mut result = Vec::with_capacity(num_iter.try_into().unwrap());

    let now = std::time::Instant::now();

    b.iter(|| {
        for i in 0..num_iter {
            let start = (i * 4) as usize;
            let end = start + 4;

            // * What I actualy want to bench.
            let array4 = safe_slice_to_array4(&v[start..end]);

            let value = u32::from_be_bytes(array4);

            result.push(value)
        }
    });

    println!("{:?}", now.elapsed());

    for i in 0..num_iter {
        assert_eq!(i, result[i as usize]);
    }
}

#[bench]
fn bench_array(b: &mut test::Bencher) {
    let listener = TcpListener::bind("127.0.0.1:6666").unwrap();
    let mut to_server = TcpStream::connect("127.0.0.1:6666").unwrap();
    let (mut to_client, _) = listener.accept().unwrap();
    let buf_write = [123; 100];
    let mut total = Vec::with_capacity(200000000);

    b.iter(|| {
        to_server.write(&buf_write).unwrap();

        let mut read_buf = [0; 4000];
        if let Ok(num) = to_client.read(&mut read_buf) {
            total.extend_from_slice(&read_buf[0..num]);
        }
    });

    println!("{}", total.len());
}

#[bench]
fn bench_vec(b: &mut test::Bencher) {
    let listener = TcpListener::bind("127.0.0.1:6666").unwrap();
    let mut to_server = TcpStream::connect("127.0.0.1:6666").unwrap();
    let (mut to_client, _) = listener.accept().unwrap();
    let buf_write = [123; 100];
    let mut total = Vec::with_capacity(200000000);

    let mut read_buf = vec![0; 4000];
    b.iter(|| {
        to_server.write(&buf_write).unwrap();

        if let Ok(num) = to_client.read(&mut read_buf) {
            total.extend_from_slice(&read_buf[0..num]);
        }
    });

    println!("{}", total.len());
}
