use std::cmp::Ordering;

use indexmap::IndexMap;
use rand::random;

fn main() {
    let mut m = IndexMap::new();

    for i in 0u32..5 {
        m.insert(i, random::<f32>());
    }
    println!("{:?}", &m);

    m.sort_by(|_, v1, _, v2| v1.partial_cmp(&v2).unwrap_or(Ordering::Equal));
    println!("{:?}", &m);
}
