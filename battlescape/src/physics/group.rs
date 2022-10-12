use super::*;

use bitflags::bitflags;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct PhysicsGroup: u32 {
        const SHIP = 1 << 0;
        const PROJECTILE = 1 << 1;
        const MISSILE = 1 << 2;
        const FIGHTER = 1 << 3;
        const SHIELD = 1 << 4;

        const ALL_TYPE = 1 << 0 | 1 << 1 | 1 << 2 | 1 << 3 | 1 << 4;

        const ALLIANCE_0 = 1 << 5;
        const ALLIANCE_1 = 1 << 6;
        const ALLIANCE_2 = 1 << 7;
        const ALLIANCE_3 = 1 << 8;
        const ALLIANCE_4 = 1 << 9;
        const ALLIANCE_5 = 1 << 10;
        const ALLIANCE_6 = 1 << 11;
        const ALLIANCE_7 = 1 << 12;

        const ALL_ALLIANCE = 1 << 5 | 1 << 6 | 1 << 7 | 1 << 8 | 1 << 9 | 1 << 10 | 1 << 11 | 1 << 12;

        const DEBRIS = 1 << 31;

        /// All of the groups.
        const ALL = u32::MAX;
        /// None of the groups.
        const NONE = 0;
    }
}
impl From<PhysicsGroup> for Group {
    fn from(value: PhysicsGroup) -> Self {
        Self::from_bits_truncate(value.bits())
    }
}
