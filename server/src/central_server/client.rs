use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(pub u64);

pub struct Client {
    fleets: AHashSet<FleetId>,
}
