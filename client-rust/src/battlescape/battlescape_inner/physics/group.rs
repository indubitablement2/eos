use super::*;
use bitflags::bitflags;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct PhysicsGroup: u32 {
        const SHIP = 1 << 0;
        const SHIELD = 1 << 1;
        const DEBRIS = 1 << 2;
        const MISSILE = 1 << 3;
        const FIGHTER = 1 << 4;
        const PROJECTILE = 1 << 5;
    }
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
