use super::*;

/// Body:
/// - HullId: u64
/// - Group ignore: 64
/// Collider:
/// - HullId: u64
/// - Is shield: 1
pub trait UserData {
    fn pack_body(hull_id: HullId, group_ignore: u64) -> Self;
    fn pack_colider(hull_id: HullId, shield: bool) -> Self;

    fn set_group_ignore(&mut self, group_ignore: u64);

    fn hull_id(self) -> HullId;
    fn hull_idx(self) -> u32;
    fn group_ignore(self) -> u64;
    fn is_shield(self) -> bool;
}
impl UserData for u128 {
    fn pack_body(hull_id: HullId, group_ignore: u64) -> Self {
        hull_id.generation.get() as u128
            | (hull_id.index as u128) << 32
            | (group_ignore as u128) << 64
    }

    fn pack_colider(hull_id: HullId, shield: bool) -> Self {
        hull_id.generation.get() as u128 | (hull_id.index as u128) << 32 | (shield as u128) << 64
    }

    fn set_group_ignore(&mut self, group_ignore: u64) {
        *self = (*self & u64::MAX as u128) | (group_ignore as u128) << 64;
    }

    fn hull_id(self) -> HullId {
        HullId {
            index: (self >> 32) as u32,
            generation: NonZeroU32::new(self as u32).unwrap(),
        }
    }

    fn hull_idx(self) -> u32 {
        (self >> 32) as u32
    }

    fn group_ignore(self) -> u64 {
        (self >> 64) as u64
    }

    fn is_shield(self) -> bool {
        self >> 64 != 0
    }
}
