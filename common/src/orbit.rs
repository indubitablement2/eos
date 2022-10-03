use serde::{Deserialize, Serialize};

// 65520, (119): [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16, 18, 20, 21, 24, 26,
// 28, 30, 35, 36, 39, 40, 42, 45, 48, 52, 56, 60, 63, 65, 70, 72, 78, 80, 84, 90, 91,
// 104, 105, 112, 117, 120, 126, 130, 140, 144, 156, 168, 180, 182, 195, 208, 210, 234,
// 240, 252, 260, 273, 280, 312, 315, 336, 360, 364, 390, 420, 455, 468, 504, 520, 546,
// 560, 585, 624, 630, 720, 728, 780, 819, 840, 910, 936, 1008, 1040, 1092, 1170, 1260,
// 1365, 1456, 1560, 1638, 1680, 1820, 1872, 2184, 2340, 2520, 2730, 3120, 3276, 3640,
// 4095, 4368, 4680, 5040, 5460, 6552, 7280, 8190, 9360, 10920, 13104, 16380, 21840, 32760]

/// All the orbit speed that will wrap around when tick is at `ORBIT_TICK_PERIOD`.
/// In radian per tick.
pub const ORBIT_SPEEDS: [f32; 120] = [
    10.0,
    5.0,
    3.3333333,
    2.5,
    2.0,
    1.6666666,
    1.4285715,
    1.25,
    1.111111,
    1.0,
    0.8333333,
    0.7692307,
    0.71428573,
    0.6666667,
    0.625,
    0.5555555,
    0.5,
    0.47619045,
    0.41666666,
    0.38461536,
    0.35714287,
    0.33333334,
    0.2857143,
    0.27777776,
    0.25641024,
    0.25,
    0.23809522,
    0.22222222,
    0.20833333,
    0.19230768,
    0.17857143,
    0.16666667,
    0.15873015,
    0.15384616,
    0.14285715,
    0.13888888,
    0.12820512,
    0.125,
    0.11904761,
    0.11111111,
    0.1098901,
    0.09615384,
    0.0952381,
    0.08928572,
    0.08547009,
    0.083333336,
    0.079365075,
    0.07692308,
    0.071428575,
    0.06944444,
    0.06410256,
    0.059523806,
    0.055555556,
    0.05494505,
    0.051282052,
    0.04807692,
    0.04761905,
    0.042735044,
    0.041666668,
    0.039682537,
    0.03846154,
    0.036630034,
    0.035714287,
    0.03205128,
    0.031746034,
    0.029761903,
    0.027777778,
    0.027472526,
    0.025641026,
    0.023809524,
    0.021978023,
    0.021367522,
    0.019841269,
    0.01923077,
    0.018315017,
    0.017857144,
    0.017094018,
    0.01602564,
    0.015873017,
    0.013888889,
    0.013736263,
    0.012820513,
    0.012210012,
    0.011904762,
    0.010989011,
    0.010683761,
    0.009920634,
    0.009615385,
    0.009157509,
    0.008547009,
    0.007936508,
    0.0073260074,
    0.0068681315,
    0.0064102565,
    0.006105006,
    0.005952381,
    0.0054945056,
    0.0053418805,
    0.0045787543,
    0.0042735045,
    0.003968254,
    0.0036630037,
    0.0032051282,
    0.003052503,
    0.0027472528,
    0.0024420025,
    0.0022893772,
    0.0021367522,
    0.001984127,
    0.0018315018,
    0.0015262516,
    0.0013736264,
    0.0012210013,
    0.0010683761,
    0.0009157509,
    0.0007631258,
    0.00061050063,
    0.00045787546,
    0.00030525032,
    0.00015262516,
];

pub fn nearest_valid_orbit_speed(orbit_speed: f32) -> f32 {
    let index = ORBIT_SPEEDS.partition_point(|other| *other < orbit_speed);

    // Get the nearest speeds.
    let small = *ORBIT_SPEEDS
        .get(index.saturating_sub(1))
        .unwrap_or(ORBIT_SPEEDS.last().unwrap());
    let big = *ORBIT_SPEEDS.get(index).unwrap_or(ORBIT_SPEEDS.last().unwrap());

    // Return which ever is closer.
    if na::ComplexField::abs(orbit_speed - small) < na::ComplexField::abs(orbit_speed - big) {
        small
    } else {
        big
    }
}

/// Number of ticks before time wrap around.
pub const ORBIT_TICK_PERIOD: u64 = 65520;

pub fn orbit_time(tick: u64) -> f32 {
    (tick % ORBIT_TICK_PERIOD) as f32 * super::METASCAPE_TICK_DURATION.as_secs_f32()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct RelativeOrbit {
    /// The distance it is orbiting from the origin.
    pub distance: f32,
    /// The initial angle.
    pub start_angle: f32,
    /// How many rad rotation does this orbit gain each second.
    ///
    /// This is also the inverse of how long it takes to make 1 rad rotation in tick.
    /// Negative value result in counter clockwise rotation.
    pub orbit_speed: f32,
}
impl RelativeOrbit {
    pub fn rotation(self, orbit_time: f32) -> f32 {
        orbit_time * self.orbit_speed + self.start_angle
    }

    /// Return the relative position of this orbit.
    pub fn to_relative_position(self, orbit_time: f32) -> na::Vector2<f32> {
        let rot = self.rotation(orbit_time);
        na::vector![na::ComplexField::cos(rot), na::ComplexField::sin(rot)] * self.distance
    }

    pub fn to_position(self, orbit_time: f32, origin: na::Vector2<f32>) -> na::Vector2<f32> {
        self.to_relative_position(orbit_time) + origin
    }

    pub fn from_relative_position(
        relative_position: na::Vector2<f32>,
        orbit_time: f32,
        distance: f32,
        orbit_speed: f32,
    ) -> Self {
        Self {
            distance,
            start_angle: orbit_time * -orbit_speed + na::RealField::atan2(relative_position.y, relative_position.x),
            orbit_speed,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Orbit {
    /// Origin in world space this orbit is orbiting aound.
    pub origin: na::Vector2<f32>,
    pub relative_orbit: RelativeOrbit,
}
impl Orbit {
    /// Return a stationary orbit at position.
    pub fn stationary(position: na::Vector2<f32>) -> Self {
        Self {
            origin: position,
            relative_orbit: RelativeOrbit::default(),
        }
    }

    pub fn from_relative_position(
        relative_position: na::Vector2<f32>,
        orbit_time: f32,
        origin: na::Vector2<f32>,
        distance: f32,
        orbit_speed: f32,
    ) -> Self {
        Self {
            origin,
            relative_orbit: RelativeOrbit::from_relative_position(relative_position, orbit_time, distance, orbit_speed),
        }
    }

    /// Return the world position of this orbit.
    pub fn to_position(self, orbit_time: f32) -> na::Vector2<f32> {
        self.relative_orbit.to_position(orbit_time, self.origin)
    }
}

#[test]
fn test_orbit() {
    use crate::rand_vector::rand_vec2;
    use rand::prelude::*;

    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let relative_position = rand_vec2(&mut rng, -100.0..100.0);
        let timef = random::<f32>() * 1000000.0;
        let o = Orbit::from_relative_position(
            relative_position,
            timef,
            na::Vector2::zeros(),
            relative_position.magnitude(),
            rng.gen::<f32>() * 0.01,
        );
        // println!(
        //     "relative pos: {:.1?}, orbit pos: {:.1?}",
        //     relative_position,
        //     o.to_position(timef)
        // );
        assert!((relative_position.abs() - o.to_position(timef).abs()).max() < 0.2);
    }
}
