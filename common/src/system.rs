use super::*;
use server::ServerId;

pub struct SystemData {
    id: u64,
    pub server_id: ServerId,
}

static DATA: std::sync::OnceLock<HashMap<u64, SystemData>> = std::sync::OnceLock::new();
fn data() -> &'static HashMap<u64, SystemData> {
    DATA.get().unwrap()
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "u64")]
#[serde(into = "u64")]
pub struct SystemId(pub &'static SystemData);
impl SystemId {
    pub fn systems_data_iter() -> std::collections::hash_map::Values<'static, u64, SystemData> {
        data().values()
    }
}
impl std::ops::Deref for SystemId {
    type Target = SystemData;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl TryFrom<u64> for SystemId {
    type Error = TryFromSystemDataIdError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        data()
            .get(&value)
            .map(Self)
            .ok_or(TryFromSystemDataIdError(value))
    }
}
impl From<SystemId> for u64 {
    fn from(idx: SystemId) -> Self {
        idx.id
    }
}
impl std::fmt::Debug for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("SystemId({})", self.id).fmt(f)
    }
}
impl std::hash::Hash for SystemId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state);
    }
}
impl PartialEq for SystemId {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl Eq for SystemId {}

pub struct TryFromSystemDataIdError(pub u64);
impl std::fmt::Display for TryFromSystemDataIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid hull data id: {} out of bound", self.0)
    }
}

pub fn load_system_data() {
    let read = std::fs::read("../client/tool/server_data/systems.json").unwrap();
    let json: Vec<SystemDataJson> = serde_json::from_slice(read.as_slice()).unwrap();
    DATA.set(
        json.into_iter()
            .map(|entity_json| entity_json.parse())
            .collect(),
    )
    .ok()
    .unwrap();
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SystemDataJson {
    id: u64,
    server_idx: u32,
}
impl SystemDataJson {
    fn parse(self) -> (u64, SystemData) {
        (
            self.id,
            SystemData {
                id: self.id,
                server_id: ServerId::try_from(self.server_idx).unwrap(),
            },
        )
    }
}
