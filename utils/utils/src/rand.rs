use rand::SeedableRng;
use rand_xoshiro::{Xoshiro128Plus, Xoshiro128StarStar};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    pub i: Xoshiro128StarStar,
    pub f: Xoshiro128Plus,
}
impl Rng {
    pub fn seed_from_u64(seed: u64) -> Self {
        let mut i = Xoshiro128StarStar::seed_from_u64(seed);
        Self { f: Xoshiro128Plus::seed_from_u64(i.gen()), i }
    }
}
impl Default for Rng {
    fn default() -> Self {
        Self::seed_from_u64(1337)
    }
}