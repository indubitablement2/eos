use super::*;
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, FromPrimitive, IntoPrimitive, Default)]
#[repr(u32)]
#[non_exhaustive]
pub enum ShipDataId {
    #[default]
    BallShip,
    CuboidShip,
}
impl ShipDataId {
    pub const fn data(self) -> ShipData {
         match self {
            Self::BallShip => ShipData {
                mobility: Mobility {
                    linear_acceleration: 1.0,
                    angular_acceleration: 1.0,
                    max_linear_velocity: 1.0,
                    max_angular_velocity: 1.0,
                },
                main_hull: HullDataId::Ball,
                auxiliary_hulls: &[],
            },
            Self::CuboidShip => ShipData {
                mobility: Mobility {
                    linear_acceleration: 1.0,
                    angular_acceleration: 1.0,
                    max_linear_velocity: 1.0,
                    max_angular_velocity: 1.0,
                },
                main_hull: HullDataId::Cuboid,
                auxiliary_hulls: &[],
            },
        }
    }
}
impl rand::distributions::Distribution<ShipDataId> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ShipDataId {
        ShipDataId::from(rng.gen_range(0..std::mem::variant_count::<ShipDataId>() as u32))
    }
}

#[derive(Debug)]
pub struct ShipData {
    pub mobility: Mobility,
    pub main_hull: HullDataId,
    // TODO: Add: init_offset.
    pub auxiliary_hulls: &'static [HullDataId],
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Mobility {
    pub linear_acceleration: f32,
    pub angular_acceleration: f32,
    pub max_linear_velocity: f32,
    pub max_angular_velocity: f32,
}
