use bevy_ecs::prelude::*;
use common::idx::*;
use glam::Vec2;
use std::ops::{Add, Sub};

//* bundle

#[derive(Bundle)]
pub struct ClientFleetBundle {
    pub client_id: ClientId,
    #[bundle]
    pub fleet_bundle: FleetBundle,
}

#[derive(Bundle)]
pub struct FleetBundle {
    pub fleet_id: FleetId,
    pub position: Position,
    pub wish_position: WishPosition,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub fleet_ai: FleetAI,
    pub reputation: Reputation,
    pub detected_radius: DetectedRadius,
    pub detector_radius: DetectorRadius,
    pub entity_detected: EntityDetected,
}

//* generic

/// The current position of the entity.
#[derive(Debug, Clone, Copy)]
pub struct Position(pub Vec2);

/// Where the entity wish to move.
#[derive(Debug, Clone, Copy)]
pub struct WishPosition(pub Vec2);

/// The current velocity of the entity.
#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Clone, Copy)]
pub struct Acceleration(pub f32);

/// Good boy points.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

//* ai

/// Not a components.
pub enum FleetGoal {
    /// Directly controlled by a client.
    Controlled,
    Trade {
        from: (),
        to: (),
    },
    Guard {
        who: Entity,
        radius: f32,
        duration: i32,
    },
    Idle {
        duration: i32,
    },
    Wandering {
        new_pos_timer: i32,
    },
}
pub struct FleetAI {
    pub goal: FleetGoal,
}

//* Detection

/// Used to make an entity detectable.
pub struct DetectedRadius(pub f32);

/// Used to detect entity that have a DetectedRadius.
pub struct DetectorRadius(pub f32);

/// Entity id that are detected by this entity.
/// If this is a client, this is sorted by entity id.
pub struct EntityDetected(pub Vec<u32>);
