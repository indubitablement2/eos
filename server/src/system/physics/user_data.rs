use super::*;

const IGNORE_GROUP_OFFSET: u32 = 16;
const TEAM_OFFSET: u32 = 32;

const IS_TINY_OFFSET: u32 = 48;
const IS_TINY_FLAG: u64 = 1 << IS_TINY_OFFSET;
const WISH_IGNORE_TINY_OFFSET: u32 = 49;
const WISH_IGNORE_TINY_FLAG: u64 = 1 << WISH_IGNORE_TINY_OFFSET;

/// - 16: entity id / collider idx
/// - 16: group
/// - 16: team
/// - 1: is_tiny
/// - 1: wish_ignore_tiny
///
/// ignoring scheme:
/// - allied fighter/missile/projectile always ignored:
/// `(is_tiny & self.team == other.team)`
///
/// - can ignore enemy fighter/missile/projectile:
/// `(is_tiny & self.ignore_tiny | other.ignore_tiny)`
///
/// - ignore an entity group:
/// `(self.group == other.group)`
#[derive(Debug, Clone, Copy)]
pub struct UserData {
    pub id: u16,
    pub ignore_group: LocalEntityId,
    pub team: PhysicsTeam,
    /// tiny == fighter | missile | projectile
    pub is_tiny: bool,
    /// Both needs needs to be tiny and wish to ignore tiny.
    pub wish_ignore_tiny: bool,
}
impl UserData {
    pub fn test(self, other: Self) -> bool {
        self.ignore_group != other.ignore_group
            && !(self.is_tiny
                && other.is_tiny
                && (other.team == self.team || (self.wish_ignore_tiny && other.wish_ignore_tiny)))
    }

    pub fn pack(self) -> u64 {
        (self.id as u64
            | (self.ignore_group as u64) << IGNORE_GROUP_OFFSET
            | (self.team as u64) << TEAM_OFFSET
            | (self.is_tiny as u64) << IS_TINY_OFFSET
            | (self.wish_ignore_tiny as u64) << WISH_IGNORE_TINY_OFFSET) as u64
    }

    pub fn unpack(user_data: u64) -> Self {
        Self {
            id: user_data as u16,
            ignore_group: (user_data >> IGNORE_GROUP_OFFSET) as u16,
            team: (user_data >> TEAM_OFFSET) as u16,
            is_tiny: (user_data & IS_TINY_FLAG) != 0,
            wish_ignore_tiny: (user_data & WISH_IGNORE_TINY_FLAG) != 0,
        }
    }

    pub fn set_user_data_id(user_data: u64, id: u16) -> u64 {
        (user_data & !(u16::MAX as u64)) | (id as u64)
    }
}
