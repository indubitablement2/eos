use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Orbit {
    /// Origin in world space this orbit is orbiting aound.
    pub origin: Vec2,
    /// The distance it is orbiting from the origin.
    pub distance: f32,
    /// The initial angle.
    pub start_angle: f32,
    /// How long it takes to make a full orbit in tick.
    /// Negative value result in counter clockwise rotation.
    ///
    /// f32 is used to allow for more granularity.
    pub orbit_time: f32,
}
impl Orbit {
    pub const DEFAULT_ORBIT_TIME: f32 = 2400.0;

    /// Return a stationary orbit at position.
    pub fn stationary(position: Vec2) -> Self {
        Self {
            origin: position,
            distance: 0.0,
            start_angle: 0.0,
            orbit_time: Self::DEFAULT_ORBIT_TIME,
        }
    }

    /// Return the world position of this orbit.
    ///
    /// Time is an f32 to allow more granularity than tick. Otherwise `u32 as f32` will work just fine.
    pub fn to_position(self, time: f32) -> Vec2 {
        if self.distance < 0.01 {
            self.origin
        } else {
            let rot = (time / self.orbit_time).mul_add(TAU, self.start_angle);
            Vec2::new(rot.cos(), rot.sin()) * self.distance + self.origin
        }
    }
}
impl Default for Orbit {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            distance: 0.0,
            start_angle: 0.0,
            orbit_time: Self::DEFAULT_ORBIT_TIME,
        }
    }
}

#[test]
fn test_orbit() {
    let t = 15.0;
    let o = Orbit {
        origin: Vec2::ZERO,
        distance: 10.0,
        start_angle: 0.0,
        orbit_time: 60.0,
    };
    let r = (-10.0f32).atan2(0.0);
    let or = Orbit {
        origin: Vec2::ZERO,
        distance: 10.0,
        start_angle: r,
        orbit_time: 60.0,
    };

    println!("{:.1?}", o.to_position(0.0));
    println!("{:.1?}", o.to_position(t));

    println!("{:.1?}", or.to_position(0.0));
    println!("{:.1?}", or.to_position(t));
}
