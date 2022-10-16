use super::*;
use bitflags::bitflags;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct PhysicsGroup: u32 {
        const SHIP = 1 << 0;
        /// Body attached to a ship.
        const SHIP_AUXILIARY = 1 << 1;
        const SHIELD = 1 << 2;
        const DEBRIS = 1 << 3;
        const MISSILE = 1 << 4;
        const FIGHTER = 1 << 5;
        const PROJECTILE = 1 << 6;
    }
}
impl PhysicsGroup {
    pub const DEFAULT_SHIP_FILTER: Self = Self::all();
}

impl Default for PhysicsGroup {
    fn default() -> Self {
        Self::all()
    }
}

impl From<PhysicsGroup> for Group {
    fn from(value: PhysicsGroup) -> Self {
        Self::from_bits_truncate(value.bits())
    }
}
