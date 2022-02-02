use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct RelativeOrbit {
    /// The distance it is orbiting from the origin.
    pub distance: f32,
    /// The initial angle.
    pub start_angle: f32,
    /// How many rad rotation does this orbit gain each time unit.
    ///
    /// This is also the inverse of how long it takes to make 1 rad rotation in tick.
    /// Negative value result in counter clockwise rotation.
    pub orbit_speed: f32,
}
impl RelativeOrbit {
    pub fn rotation(self, timef: f32) -> f32 {
        timef.mul_add(self.orbit_speed, self.start_angle)
    }

    /// Return the relative position of this orbit.
    ///
    /// `timef` is an f32 to allow more granularity than tick. Otherwise `u32 as f32` will work just fine.
    pub fn to_relative_position(self, time: f32) -> Vec2 {
        let rot = self.rotation(time);
        Vec2::new(rot.cos(), rot.sin()) * self.distance
    }

    pub fn to_position(self, timef: f32, origin: Vec2) -> Vec2 {
        self.to_relative_position(timef) + origin
    }

    pub fn from_relative_position(relative_position: Vec2, timef: f32, distance: f32, orbit_speed: f32) -> Self {
        Self {
            distance,
            start_angle: timef.mul_add(-orbit_speed, relative_position.y.atan2(relative_position.x)),
            orbit_speed,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Orbit {
    /// Origin in world space this orbit is orbiting aound.
    pub origin: Vec2,
    pub relative_orbit: RelativeOrbit,
}
impl Orbit {
    /// Return a stationary orbit at position.
    pub fn stationary(position: Vec2) -> Self {
        Self {
            origin: position,
            relative_orbit: RelativeOrbit::default(),
        }
    }

    pub fn from_relative_position(
        relative_position: Vec2,
        timef: f32,
        origin: Vec2,
        distance: f32,
        orbit_speed: f32,
    ) -> Self {
        Self {
            origin,
            relative_orbit: RelativeOrbit::from_relative_position(relative_position, timef, distance, orbit_speed),
        }
    }

    /// Return the world position of this orbit.
    ///
    /// `timef` is an f32 to allow more granularity than tick. Otherwise `u32 as f32` will work just fine.
    pub fn to_position(self, timef: f32) -> Vec2 {
        self.relative_orbit.to_position(timef, self.origin)
    }
}

#[test]
fn test_orbit() {
    use rand::random;
    for _ in 0..10 {
        let relative_position = random::<Vec2>() * 200.0 - 100.0;
        let timef = random::<f32>() * 1000000.0;
        let o = Orbit::from_relative_position(
            relative_position,
            timef,
            Vec2::ZERO,
            relative_position.length(),
            random::<f32>() * 0.01,
        );
        println!(
            "relative pos: {:.1?}, orbit pos: {:.1?}",
            relative_position,
            o.to_position(timef)
        );
        assert!(relative_position.abs_diff_eq(o.to_position(timef), 0.2));
    }
}
