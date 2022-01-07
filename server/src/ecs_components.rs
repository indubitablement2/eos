use ahash::AHashMap;
use bevy_ecs::prelude::*;
use common::idx::*;
use glam::Vec2;
use std::{
    collections::VecDeque,
    ops::{Add, Sub},
};

// TODO: impl component for these when 0.6 release.
// Orbit

//* bundle

#[derive(Bundle)]
pub struct ClientFleetBundle {
    pub client_id: ClientId,
    pub know_entities: KnowEntities,
    #[bundle]
    pub fleet_bundle: FleetBundle,
}

#[derive(Bundle)]
pub struct FleetBundle {
    pub name: Name,
    pub fleet_id: FleetId,
    pub position: Position,
    pub wish_position: WishPosition,
    pub velocity: Velocity,
    pub derived_fleet_stats: DerivedFleetStats,
    pub reputation: Reputation,
    pub detected_radius: DetectedRadius,
    pub detector_radius: DetectorRadius,
    pub entity_detected: EntityDetected,
}

//* Client

/// Entity we have sent informations to the client.
///
/// Instead of sending the whole entity id, we identify entities with a temporary 8bits id.
#[derive(Debug, Default)]
pub struct KnowEntities {
    pub free_idx: VecDeque<u8>,
    pub known: AHashMap<Entity, u8>,
}

// * Generic

/// A standard position relative to the world origin.
pub struct Position(pub Vec2);

/// An entity's display name.
pub struct Name(pub String);

//* Fleet

/// Where the fleet wish to move.
#[derive(Debug, Clone, Copy, Default)]
pub struct WishPosition(pub Option<Vec2>);

/// The current velocity of the entity.
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec2);

/// Fleet statistics that are derived from fleet composition.
#[derive(Debug, Clone, Copy)]
pub struct DerivedFleetStats {
    /// How much velocity this entity can gain each update.
    pub acceleration: f32,
    /// Velocity beyong which it can not accelerate itself anymore (it can still be pushed).
    pub max_speed: f32,
}

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

//* AI

/// Not a components.
#[derive(Debug, Clone, Copy)]
pub enum ColonyFleetAIGoal {
    Trade { colony: Entity },
    Guard { duration: i32 },
}
/// Ai for fleet that are owned by a colony.
pub struct ColonyFleetAI {
    pub goal: ColonyFleetAIGoal,
    /// The colony that own this fleet.
    pub colony: Entity,
}

/// Not a components.
#[derive(Debug, Clone, Copy, Default)]
pub enum ClientFleetAIGoal {
    #[default]
    Idle,
    Flee,
}
#[derive(Debug, Default, Clone, Copy)]
/// Ai that takes over a client's fleet when it is not connected.
pub struct ClientFleetAI {
    pub goal: ClientFleetAIGoal,
}

//* Detection

/// Used to make an entity detectable.
pub struct DetectedRadius(pub f32);

/// Used to detect entity that have a DetectedRadius.
pub struct DetectorRadius(pub f32);

/// Entity id that are detected by this entity.
/// If this is a client, this is sorted by entity id.
pub struct EntityDetected(pub Vec<u32>);
