use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RangeI32 {
    pub min: i32,
    pub extra: i32,
}
impl RangeI32 {
    pub fn new(min: i32, max: i32) -> Self {
        Self {
            min,
            extra: (max - min).max(1),
        }
    }

    pub fn roll(&self, rand: i32) -> i32 {
        self.min + (rand.abs() % self.extra)
    }
}

#[test]
fn test_rand() {
    let min = 1;
    let max = 3;
    let r = RangeI32::new(min, max);

    for _ in 0..100 {
        let v =  r.roll(rand::random());
        print!("{}, ", &v);
        assert!(v >= min && v < max);
    }
    println!();
}