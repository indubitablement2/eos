use std::hash::{Hash, Hasher};

fn main() {
    let mut data = vec![
        MiniData {
            first: "hello!".to_string(),
            seconds: "123".to_string(),
        },
        MiniData {
            first: "Ok!".to_string(),
            seconds: "Last".to_string(),
        },
    ];

    let mut hasher1 = ahash::AHasher::new_with_keys(1667, 420);
    data.hash(&mut hasher1);
    let mut hasher2 = ahash::AHasher::new_with_keys(1667, 420);
    data.reverse().hash(&mut hasher2);

    println!("{:x}, {:x}", hasher1.finish(), hasher2.finish());
}

#[derive(Debug, Hash)]
struct MiniData {
    pub first: String,
    pub seconds: String,
}
