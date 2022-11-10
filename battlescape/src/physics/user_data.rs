use super::*;

pub struct Hooks;
impl PhysicsHooks for Hooks {
    fn filter_contact_pair(&self, context: &PairFilterContext) -> Option<SolverFlags> {
        let a = context.colliders[context.collider1].user_data;
        let b = context.colliders[context.collider2].user_data;
        if UserData::filter(a, b) {
            Some(SolverFlags::COMPUTE_IMPULSES)
        } else {
            None
        }
    }

    fn filter_intersection_pair(&self, context: &PairFilterContext) -> bool {
        let a = context.colliders[context.collider1].user_data;
        let b = context.colliders[context.collider2].user_data;
        UserData::filter(a, b)
    }

    fn modify_solver_contacts(&self, _context: &mut ContactModificationContext) {}
}

/// from low to high bits in chunk of 32 bits:
/// ### team
/// If `ignore_team` bit is set, ignore all collider with the same team.
/// Only one collider need to have this to take effect.
/// ### group ignore
/// Ignore all collider with the same group.
/// Default to creating a new group so that other collider can ignore us.
/// ### id
/// ### bitfield/id type
/// - 0..8: id type
/// - 32-1: ignore_team
pub struct UserData;
impl UserData {
    const TEAM_MASK: u128 = u32::MAX as u128;

    const GROUP_IGNORE_OFFSET: u32 = u32::BITS;
    const GROUP_IGNORE_MASK: u128 = (u32::MAX as u128) << Self::GROUP_IGNORE_OFFSET;

    const ID_OFFSET: u32 = u32::BITS + Self::GROUP_IGNORE_OFFSET;
    // const ID_MASK: u128 = (u32::MAX as u128) << Self::ID_OFFSET;

    const ID_TYPE_OFFSET: u32 = u32::BITS + Self::ID_OFFSET;
    // const ID_TYPE_MASK: u128 = u8::MAX << Self::ID_TYPE_OFFSET;

    const BITMASK_IGNORE_TEAM: u128 = 1 << (u128::BITS - 1);

    /// If true, this collider ignore all collider with the same team.
    pub fn set_team(user_data: u128, team: u32) -> u128 {
        (user_data & !Self::TEAM_MASK) | team as u128
    }

    /// Ignore all collider with the same group.
    pub fn set_group_ignore(user_data: u128, group_ignore: u32) -> u128 {
        (user_data & !Self::GROUP_IGNORE_MASK)
            | ((group_ignore as u128) << Self::GROUP_IGNORE_OFFSET)
    }

    pub fn build(team: u32, group_ignore: u32, id: GenericId, ignore_team: bool) -> u128 {
        let mut bits = team as u128;

        bits |= (group_ignore as u128) << Self::GROUP_IGNORE_OFFSET;

        let (id_type, id) = id.pack();
        bits |= (id as u128) << Self::ID_OFFSET;
        bits |= (id_type as u128) << Self::ID_TYPE_OFFSET;

        if ignore_team {
            bits |= Self::BITMASK_IGNORE_TEAM;
        }

        bits
    }

    /// Return true if these `user_data` can interact.
    pub fn filter(a: u128, b: u128) -> bool {
        let a_team = a as u32;
        let b_team = b as u32;
        let a_team_ignore = a & Self::BITMASK_IGNORE_TEAM != 0;
        let b_team_ignore = b & Self::BITMASK_IGNORE_TEAM != 0;
        let a_group_ignore = (a >> Self::GROUP_IGNORE_OFFSET) as u32;
        let b_group_ignore = (b >> Self::GROUP_IGNORE_OFFSET) as u32;

        !(a_team == b_team && (a_team_ignore || b_team_ignore)) && a_group_ignore != b_group_ignore
    }

    pub fn id(user_data: u128) -> GenericId {
        let id = (user_data >> Self::ID_OFFSET) as u32;
        let id_type = (user_data >> Self::ID_TYPE_OFFSET) as u8;
        GenericId::unpack(id_type, id)
    }
}

#[test]
pub fn test_user_data() {
    // Same team, but not set to ignore.
    assert!(UserData::filter(
        UserData::build(0, 0, GenericId::ShipId(Default::default()), false),
        UserData::build(0, 1, GenericId::ShipId(Default::default()), false)
    ));

    // Same team, but not all set to ignore.
    assert!(!UserData::filter(
        UserData::build(0, 0, GenericId::ShipId(Default::default()), false),
        UserData::build(0, 1, GenericId::ShipId(Default::default()), true)
    ));

    // Same team, but set to ignore.
    assert!(!UserData::filter(
        UserData::build(0, 0, GenericId::ShipId(Default::default()), true),
        UserData::build(0, 1, GenericId::ShipId(Default::default()), true)
    ));

    // Same group ignore.
    assert!(!UserData::filter(
        UserData::build(0, 0, GenericId::ShipId(Default::default()), false),
        UserData::build(1, 0, GenericId::ShipId(Default::default()), false)
    ));
}
