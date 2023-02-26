use super::*;

/// Possible id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenericId {
    Entity(EntityId),
    Shield(EntityId),
    Projectile(u32), // TODO: Just a test id.
}
impl GenericId {
    pub fn pack(self) -> u128 {
        let (id_type, id) = match self {
            GenericId::Entity(id) => (0, id.0 as u128),
            GenericId::Shield(id) => (1, id.0 as u128),
            GenericId::Projectile(id) => (2, id as u128),
        };

        id_type | id << 32
    }

    pub fn unpack(data: u128) -> Self {
        let id_type = data as u32;
        let id = (data >> 32) as u32;
        match id_type {
            0 => Self::Entity(EntityId(id)),
            1 => Self::Shield(EntityId(id)),
            _ => Self::Projectile(id),
        }
    }
}

/// - id type: 32
/// - id: 32
/// - unused: 32
/// - group ignore: 32
pub trait UserData {
    fn pack(id: GenericId, group_ignore: GroupIgnore) -> Self;
    fn id(self) -> GenericId;
    fn group_ignore(self) -> GroupIgnore;
}
impl UserData for u128 {
    fn pack(id: GenericId, group_ignore: GroupIgnore) -> Self {
        id.pack() | (group_ignore as u128) << 96
    }

    fn id(self) -> GenericId {
        GenericId::unpack(self)
    }

    fn group_ignore(self) -> GroupIgnore {
        (self >> 96) as GroupIgnore
    }
}
