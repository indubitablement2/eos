use self::simulation::HullDataId;

use super::*;
use ids::Id;
use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ShipId(NonZeroU64);
impl Default for ShipId {
    fn default() -> Self {
        Self(NonZeroU64::MIN)
    }
}
impl Id for ShipId {
    fn next(&mut self) -> Self {
        let ret = *self;
        self.0 = self.0.checked_add(1).unwrap();
        ret
    }

    fn to_u64(&self) -> u64 {
        self.0.get()
    }

    fn try_from_u64(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }
}

pub struct ShipData {
    idx: u64,
    pub hull_data_id: HullDataId,
}

static DATA: std::sync::OnceLock<Vec<ShipData>> = std::sync::OnceLock::new();

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "u64")]
#[serde(into = "u64")]
pub struct ShipDataId(pub &'static ShipData);
impl ShipDataId {
    pub fn data() -> &'static [ShipData] {
        DATA.get().unwrap()
    }
}
impl std::ops::Deref for ShipDataId {
    type Target = ShipData;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl TryFrom<u64> for ShipDataId {
    type Error = u64;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        ShipDataId::data()
            .get(value as usize)
            .map(Self)
            .ok_or(value)
    }
}
impl From<ShipDataId> for u64 {
    fn from(id: ShipDataId) -> Self {
        id.idx
    }
}
impl std::fmt::Debug for ShipDataId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShipDataId({})", self.idx)
    }
}
impl std::hash::Hash for ShipDataId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state);
    }
}
impl PartialEq for ShipDataId {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl Eq for ShipDataId {}
impl Default for ShipDataId {
    fn default() -> Self {
        ShipDataId::data().first().map(Self).unwrap()
    }
}

pub fn load_data() {
    let read = std::fs::read("../client/tool/server_data/ships.json").unwrap();
    let json: Vec<ShipDataJson> = serde_json::from_slice(read.as_slice()).unwrap();
    DATA.set(
        json.into_iter()
            .zip(0..)
            .map(|(json, idx)| json.parse(idx))
            .collect(),
    )
    .ok()
    .unwrap();
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ShipDataJson {
    hull_idx: u32,
}
impl ShipDataJson {
    fn parse(self, idx: u64) -> ShipData {
        ShipData {
            idx,
            hull_data_id: HullDataId::try_from(self.hull_idx).unwrap(),
        }
    }
}
