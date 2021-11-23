use crate::collision::ColliderId;
use crate::res_clients::ClientId;
use glam::Vec2;

/// The current position of the entity.
#[derive(Debug, Clone, Copy)]
pub struct Position(pub Vec2);

/// Where the entity wish to move.
#[derive(Debug, Clone, Copy)]
pub struct WishPosition(pub Vec2);

/// The current velocity of the entity.
#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

/// For some time, prevent despawning when outside a reality bubble.
#[derive(Debug, Clone, Copy)]
pub struct NoDespawnTimer(pub i32);

/// Entity is directly controlled by this client.
#[derive(Debug, Clone, Copy)]
pub struct Controlled(pub ClientId);

pub struct FleetCollider(ColliderId);
impl FleetCollider {
    pub const RADIUS_MAX: f32 = 128.0;
}
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
