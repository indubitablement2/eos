use crate::collision::ColliderId;
use std::ops::{Add, Sub};
use bevy_ecs::prelude::*;
use glam::Vec2;

//* bundle */

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
    pub fleet_collider: FleetCollider,
    pub reputation: Reputation,
    pub fleet_detection_radius: FleetDetectionRadius,
    pub fleet_detected: FleetDetected,
}

//* id */

/// 0 is reserved and mean unvalid/server.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientId(pub u32);
impl ClientId {
    /// Return if this is a valid ClientId, id != 0.
    pub fn is_valid(self) -> bool {
        self.0 != 0
    }
}
impl From<FleetId> for ClientId {
    fn from(fleet_id: FleetId) -> Self {
        Self(fleet_id.0 as u32)
    }
}

/// Never recycled.
/// First 2^32 - 1 idx are reserved for clients.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FleetId(pub u64);
impl From<ClientId> for FleetId {
    fn from(client_id: ClientId) -> Self {
        Self(client_id.0 as u64)
    }
}

/// Never recycled.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FactionId(u32);

//* generic */

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

/// Radius that is tested agains fleet collider to determine if this fleet can see another fleet.
pub struct FleetDetectionRadius(pub f32);

/// Fleets that are detected by this fleet.
pub struct FleetDetected(pub Vec<FleetId>);

//* ai */

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

//* collider */

/// Used to detect a fleet.
pub struct FleetCollider(pub ColliderId);
impl FleetCollider {
    pub const RADIUS_MAX: f32 = 128.0;
}

/// Used to enter a system.
pub struct SystemCollider(ColliderId);
impl SystemCollider {
    pub const RADIUS_MIN: f32 = 64.0;
    pub const RADIUS_MAX: f32 = 256.0;
}

pub struct RealityBubbleCollider(ColliderId);
impl RealityBubbleCollider {
    pub const RADIUS: f32 = 256.0;
}

pub struct FactionActivityCollider(ColliderId);
impl FactionActivityCollider {
    pub const RADIUS_MAX: f32 = 128.0;
}
