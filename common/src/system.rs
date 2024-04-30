use super::*;

pub struct SystemData {
    id: u32,
}

static DATA: std::sync::OnceLock<Vec<SystemData>> = std::sync::OnceLock::new();

fn data() -> &'static [SystemData] {
    DATA.get().unwrap()
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "u32")]
#[serde(into = "u32")]
pub struct SystemDataId(pub &'static SystemData);
impl Default for SystemDataId {
    fn default() -> Self {
        Self(data().first().unwrap())
    }
}
impl std::ops::Deref for SystemDataId {
    type Target = SystemData;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl TryFrom<u32> for SystemDataId {
    type Error = TryFromSystemDataIdError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        data()
            .get(value as usize)
            .map(Self)
            .ok_or(TryFromSystemDataIdError(value))
    }
}
impl From<SystemDataId> for u32 {
    fn from(idx: SystemDataId) -> Self {
        idx.id
    }
}
impl std::fmt::Debug for SystemDataId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

pub struct TryFromSystemDataIdError(pub u32);
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
            .zip(0u32..)
            .map(|(entity_json, id)| entity_json.parse(id))
            .collect(),
    )
    .ok()
    .unwrap();
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SystemDataJson {}
impl SystemDataJson {
    fn parse(self, id: u32) -> SystemData {
        SystemData { id }
    }
}
