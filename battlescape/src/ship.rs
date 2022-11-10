use super::*;
use common::ship_data::*;

pub type AuxiliaryHulls = SmallVec<[HullId; 4]>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Serialize, Deserialize,
)]
pub struct ShipId(pub u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct BattlescapeShip {
    pub fleet_id: FleetId,
    pub index: usize,

    pub ship_data_id: ShipDataId,
    pub rb: RigidBodyHandle,
    pub mobility: Mobility,
    pub main_hull: HullId,
    pub auxiliary_hulls: AuxiliaryHulls,
}
