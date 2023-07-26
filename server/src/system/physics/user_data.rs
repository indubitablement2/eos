use super::*;

const IGNORE_GROUP_OFFSET: u32 = 16;
const TEAM_OFFSET: u32 = 32;

const WISH_IGNORE_SAME_TEAM_OFFSET: u32 = 48;
const WISH_IGNORE_SAME_TEAM_FLAG: u64 = 1 << WISH_IGNORE_SAME_TEAM_OFFSET;
const FORCE_IGNORE_SAME_TEAM_OFFSET: u32 = 49;
const FORCE_IGNORE_SAME_TEAM_FLAG: u64 = 1 << FORCE_IGNORE_SAME_TEAM_OFFSET;

/// - 16: entity id
/// - 16: group
/// - 16: team
/// - 1: wish_ignore_same_team
/// - 1: force_ignore_same_team
///
/// ignoring scheme:
/// - allied fighter/missile/projectile always ignored:
/// `(self.wish_ignore_same_team && other.wish_ignore_same_team && self.team == other.team)`
///
/// - ignore an entity group:
/// `(self.group == other.group)`
#[derive(Debug, Clone, Copy)]
pub struct UserData {
    pub id: LocalEntityId,
    pub ignore_group: LocalEntityId,
    pub team: PhysicsTeam,
    pub wish_ignore_same_team: bool,
    pub force_ignore_same_team: bool,
}
impl UserData {
    pub fn test(self, other: Self) -> bool {
        self.ignore_group != other.ignore_group
            && !(self.team == other.team
                && ((self.wish_ignore_same_team && other.wish_ignore_same_team)
                    || self.force_ignore_same_team
                    || other.force_ignore_same_team))
    }

    pub fn pack(self) -> u64 {
        (self.id as u64
            | (self.ignore_group as u64) << IGNORE_GROUP_OFFSET
            | (self.team as u64) << TEAM_OFFSET
            | (self.wish_ignore_same_team as u64) << WISH_IGNORE_SAME_TEAM_OFFSET
            | (self.force_ignore_same_team as u64) << FORCE_IGNORE_SAME_TEAM_OFFSET) as u64
    }

    pub fn unpack(user_data: u64) -> Self {
        Self {
            id: user_data as u16,
            ignore_group: (user_data >> IGNORE_GROUP_OFFSET) as u16,
            team: (user_data >> TEAM_OFFSET) as u16,
            wish_ignore_same_team: (user_data & WISH_IGNORE_SAME_TEAM_FLAG) != 0,
            force_ignore_same_team: (user_data & FORCE_IGNORE_SAME_TEAM_FLAG) != 0,
        }
    }

    pub fn set_user_data_id(user_data: u64, id: u16) -> u64 {
        (user_data & !(u16::MAX as u64)) | (id as u64)
    }
}
