use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Add, Sub},
};

pub enum ReputationState {
    Allied,
    Neutral,
    Enemy,
}

/// Good boy points.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Reputation(i8);
impl Reputation {
    pub const NEUTRAL: Reputation = Reputation(0);
    pub const ENEMY_THRESHOLD: Reputation = Reputation(-25);
    pub const ALLIED_THRESHOLD: Reputation = Reputation(25);
    pub const MIN: Reputation = Self(-100);
    pub const MAX: Reputation = Self(100);

    pub fn from_raw(reputation: i8) -> Self {
        Self(reputation)
    }

    pub fn is_allied(self) -> bool {
        self > Self::ALLIED_THRESHOLD
    }

    pub fn is_enemy(self) -> bool {
        self < Self::ENEMY_THRESHOLD
    }

    pub fn get_reputation_state(self) -> ReputationState {
        if self.is_allied() {
            ReputationState::Allied
        } else if self.is_enemy() {
            ReputationState::Enemy
        } else {
            ReputationState::Neutral
        }
    }
}
impl Add for Reputation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0)).min(Self::MAX)
    }
}
impl Sub for Reputation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0)).max(Self::MIN)
    }
}
impl Display for Reputation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
