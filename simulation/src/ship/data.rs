use super::*;

#[derive(Debug)]
pub struct ShipData {
    pub mobility: Mobility,
    // TODO: Add: init_offset.
    /// First hull is main.
    /// Can not add/remove hulls after creation.
    pub hulls: &'static [HullDataId],
    // TODO: weapon slot
    // TODO: built-in weapon (take a slot #)
    // TODO: Engine placement
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, FromPrimitive, IntoPrimitive, Default,
)]
#[non_exhaustive]
#[repr(u32)]
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
                    angular_acceleration: 0.5,
                    max_linear_velocity: 7.0,
                    max_angular_velocity: 3.0,
                },
                hulls: &[HullDataId::Ball],
            },
            Self::CuboidShip => ShipData {
                mobility: Mobility {
                    linear_acceleration: 1.0,
                    angular_acceleration: 0.5,
                    max_linear_velocity: 7.0,
                    max_angular_velocity: 3.0,
                },
                hulls: &[HullDataId::Cuboid],
            },
        }
    }
}
impl rand::distributions::Distribution<ShipDataId> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ShipDataId {
        ShipDataId::from(rng.gen_range(0..std::mem::variant_count::<ShipDataId>() as u32))
    }
}
