use std::ops::{Add, Sub};
use serde::{Serialize, Deserialize};

/// Good boy points.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Reputation(pub i8);
impl Reputation {
    const ALLIED_THRESHOLD: i8 = 30;
    const ENEMY_THRESHOLD: i8 = -30;

    pub fn is_ally(self) -> bool {
        self.0 > Reputation::ALLIED_THRESHOLD
    }

    pub fn is_enemy(self) -> bool {
        self.0 < Reputation::ENEMY_THRESHOLD
    }
}
impl Default for Reputation {
    fn default() -> Self {
        Self(0)
    }
}
impl Add for Reputation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}
impl Sub for Reputation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}