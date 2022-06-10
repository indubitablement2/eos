use soa_derive::Soa;
use utils::*;

#[derive(Soa)]
pub struct Test {
    a: usize,
    b: String,
    buffer: Vec<u8>,
}

fn main() {
    let mut test_soa = TestSoa::new();

    test_soa.push(Test {
        a: 42,
        b: "Hello".to_string(),
        buffer: "world!".as_bytes().to_vec(),
    });

    for b in test_soa.b.iter() {
        println!("{}", b);
    }
}
