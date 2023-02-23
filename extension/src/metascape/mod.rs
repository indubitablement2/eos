pub mod fleet;
pub mod ship;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FleetId(pub u64);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct ClientId(pub u64);

/// 0 is intended for local only bs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct BattlescapeId(pub u64);

pub struct Metascape {
    fleet: fleet::Fleet,
}
