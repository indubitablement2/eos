use super::*;

/// Possible id of a rigid body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyGenericId {
    EntityId(EntityId),
}

/// Possible id of a collider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColliderGenericId {
    Hull { entity_id: EntityId },
}

/// - id: 64
/// - id type: 4
/// - group_ignore 60
pub trait UserData {
    const ID_TYPE_OFFSET: u32 = u64::BITS;
    const GROUP_IGNORE_OFFSET: u32 = Self::ID_TYPE_OFFSET + 4;
    fn pack_body(id: BodyGenericId, group_ignore: GroupIgnore) -> Self;
    fn pack_collider(id: ColliderGenericId, group_ignore: GroupIgnore) -> Self;
    fn id_body(self) -> BodyGenericId;
    fn id_collider(self) -> ColliderGenericId;
    fn group_ignore(self) -> GroupIgnore;
}
impl UserData for u128 {
    const ID_TYPE_OFFSET: u32 = u64::BITS;
    const GROUP_IGNORE_OFFSET: u32 = Self::ID_TYPE_OFFSET + 4;

    fn pack_body(id: BodyGenericId, group_ignore: GroupIgnore) -> Self {
        let id = match id {
            BodyGenericId::EntityId(id) => 0 << u64::BITS | id.0 as u128,
        };
        id | (group_ignore as u128) << Self::GROUP_IGNORE_OFFSET
    }

    fn pack_collider(id: ColliderGenericId, group_ignore: GroupIgnore) -> Self {
        let id = match id {
            ColliderGenericId::Hull { entity_id } => 0 << u64::BITS | (entity_id.0 as u128),
        };
        id | (group_ignore as u128) << Self::GROUP_IGNORE_OFFSET
    }

    fn id_body(self) -> BodyGenericId {
        match (self >> u64::BITS) & 0b1111 {
            0 => BodyGenericId::EntityId(EntityId(self as u32)),
            _ => unreachable!(),
        }
    }

    fn id_collider(self) -> ColliderGenericId {
        match (self >> u64::BITS) & 0b1111 {
            0 => ColliderGenericId::Hull {
                entity_id: EntityId(self as u32),
            },
            _ => unreachable!(),
        }
    }

    fn group_ignore(self) -> GroupIgnore {
        (self >> Self::GROUP_IGNORE_OFFSET) as u64
    }
}
