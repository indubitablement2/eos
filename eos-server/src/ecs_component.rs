use eos_common::{idx::ClientId, location::*};
use glam::Vec2;

pub struct ClientIdComp(pub ClientId);

pub struct LocationComp(pub Location);

/// System the entity is inside of.
pub struct SystemComp(pub usize);

/// Current fleet velocity.
pub struct VelocityComp(pub Vec2);

/// Go toward a location.
// pub struct WishLocationComp(pub Location);

/// Pursue another fleet.
// TODO
pub struct WishPursueComp();

pub struct MovementStateComp(pub MovementState);

pub enum MovementState {
    /// Rotating around a planet or star.
    Orbiting(f32),
    /// Breaking to 0 speed.
    /// Value is used to modify breaking speed.
    Breaking(f32),
    /// Trying to reach max speed toward target.
    /// Value is vector to target.
    Seeking(Vec2),
    /// Going at max speed toward its target with no external force. Used on long inter system travel.
    /// Value is vector to target.
    Cruising(Vec2),
    /// No velocity. If inside a system, also looking for an object to orbit.
    Still,
}
