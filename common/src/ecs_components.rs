use crate::collision::ColliderId;
use glam::Vec2;

pub struct Position(Vec2);
pub struct Velocity(Vec2);
/// For some time, prevent despawning when outside a reality bubble.
pub struct NoDespawnTimer(i32);


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
