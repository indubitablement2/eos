use rand::{distributions::Uniform, Rng};

fn main() {
    let mut rng = rand::thread_rng();

    // Get a single i32 from a range.
    println!("{}", rng.gen_range(0i32..5));

    // Get 10 i32 from a range.
    let v: Vec<i32> = (&mut rng).sample_iter(Uniform::new(0i32, 5)).take(10).collect();
    println!("{:?}", v);

    // Try to get an i32 where range min > max.
    let mut r = 5i32..10;
    if r.start >= r.end {
        r.end = r.start + 1;
    }
    println!("{}", rng.gen_range(r.clone()));

    let s = bincode::serialize(&r).unwrap();
    println!("{:?}", s);
}
