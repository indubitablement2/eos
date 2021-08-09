use crate::const_var::*;
use glam::{ivec2, IVec2};
use serde::{Deserialize, Serialize};

/// Unique client identifier. Correspond to its SteamId.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct ClientId(pub u64);

impl ClientId {
    /// Return if this is a valid id. Aka: not ClientId(0).
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// Unique fleet identifier. Id are never recycled. u64 guarantees that no duplicate will ever be created.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct FleetId(pub u64);

/// Unique sector identifier. Also correspond to its position in SpaceGrid.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct SectorId(pub u16);

impl SectorId {
    pub fn to_ivec(&self) -> IVec2 {
        ivec2(self.0 as i32 % X_SECTOR, self.0 as i32 / X_SECTOR)
    }
}

// /// Unique system identifier. Also correspond to its position in Sector.
// #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, Hash)]
// pub struct SystemId(pub SectorId, u8);
