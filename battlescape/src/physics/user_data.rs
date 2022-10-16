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

// from high to low bits in chunk of 32
// - team
// - team ignore
// - ignore rb generation
// - ignore rb id
pub struct UserData;
impl UserData {
    const IGNORE_RB_ID_MASK: u128 = u32::MAX as u128;
    const IGNORE_RB_GEN_OFFSET: u32 = u32::BITS;
    const IGNORE_RB_GEN_MASK: u128 = (u32::MAX as u128) << Self::IGNORE_RB_GEN_OFFSET;
    const IGNORE_RB_MASK: u128 = Self::IGNORE_RB_ID_MASK | Self::IGNORE_RB_GEN_MASK;

    const IGNORE_TEAM_OFFSET: u32 = u64::BITS;
    const IGNORE_TEAM_MASK: u128 = (u32::MAX as u128) << Self::IGNORE_TEAM_OFFSET;

    const TEAM_OFFSET: u32 = (Self::IGNORE_TEAM_OFFSET + u32::BITS);
    const TEAM_MASK: u128 = (u32::MAX as u128) << Self::TEAM_OFFSET;

    /// Will ignore any collider with this rigitd body handle.
    ///
    /// If you don't want to ignore any collider, set to your own rb handle so that other can ignore you.
    /// Or, **for sensor only**, set to None.
    pub fn set_rb_ignore(user_data: u128, rb_ignore: Option<RigidBodyHandle>) -> u128 {
        if let Some(rb_ignore) = rb_ignore {
            let user_data = user_data & !Self::IGNORE_RB_MASK;
            let (id, generation) = rb_ignore.into_raw_parts();
            user_data | ((generation as u128) << Self::IGNORE_RB_GEN_OFFSET) | id as u128
        } else {
            user_data | Self::IGNORE_RB_MASK
        }
    }

    pub fn get_rb_ignore(user_data: u128) -> RigidBodyHandle {
        let data = user_data as u64;
        RigidBodyHandle::from_raw_parts(data as u32, (data >> Self::IGNORE_RB_GEN_OFFSET) as u32)
    }

    /// Will ignore any collider with this team.
    pub fn set_team_ignore(user_data: u128, team_ignore: Option<u32>) -> u128 {
        if let Some(team_ignore) = team_ignore {
            let user_data = user_data & !Self::IGNORE_TEAM_MASK;
            user_data | ((team_ignore as u128) << Self::IGNORE_TEAM_OFFSET)
        } else {
            user_data | Self::IGNORE_TEAM_MASK
        }
    }

    /// Set to your team so that other collider can ignore you.
    pub fn set_team(user_data: u128, team: Option<u32>) -> u128 {
        if let Some(team) = team {
            let user_data = user_data & !Self::TEAM_MASK;
            user_data | ((team as u128) << Self::TEAM_OFFSET)
        } else {
            user_data | Self::TEAM_MASK
        }
    }

    pub fn build(
        rb_ignore: Option<RigidBodyHandle>,
        team_ignore: Option<u32>,
        team: Option<u32>,
    ) -> u128 {
        Self::set_team(
            Self::set_team_ignore(Self::set_rb_ignore(0, rb_ignore), team_ignore),
            team,
        )
    }

    /// Return true if those `user_data` can interact.
    pub fn filter(a: u128, b: u128) -> bool {
        let a_rb = a as u64;
        let b_rb = b as u64;
        let a_team = (a >> Self::TEAM_OFFSET) as u32;
        let a_team_ignore = (a >> Self::IGNORE_TEAM_OFFSET) as u32;
        let b_team = (b >> Self::TEAM_OFFSET) as u32;
        let b_team_ignore = (b >> Self::IGNORE_TEAM_OFFSET) as u32;

        (a_rb != b_rb || a_rb == u64::MAX || b_rb == u64::MAX)
            && ((a_team_ignore != b_team && b_team_ignore != a_team)
                || a_team == u32::MAX
                || b_team == u32::MAX)
    }
}

#[test]
pub fn test_user_data() {
    let rb_a = RigidBodyHandle::from_raw_parts(0, 0);
    let rb_b = RigidBodyHandle::from_raw_parts(1, 0);

    let a = UserData::build(Some(rb_a), Some(0), Some(0));
    let b = UserData::build(Some(rb_b), Some(1), Some(1));
    assert!(UserData::filter(a, b));
    let c = UserData::build(None, Some(0), Some(0));
    assert!(!UserData::filter(a, c));
    assert!(UserData::filter(b, c));
}
