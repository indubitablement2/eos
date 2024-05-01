use self::system::SystemId;

use super::*;
use std::ops::{Index, IndexMut};

// Loaded before system data.
pub struct ServerData {
    pub idx: usize,
    /// Where clients connect to.
    pub ws_addr: String,
}

static DATA: std::sync::OnceLock<Vec<ServerData>> = std::sync::OnceLock::new();

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "u32")]
#[serde(into = "u32")]
pub struct ServerId(pub &'static ServerData);
impl ServerId {
    pub fn data() -> &'static [ServerData] {
        DATA.get().unwrap()
    }

    /// Returns the systems that are handled by this server.
    pub fn systems(self) -> Vec<SystemId> {
        SystemId::systems_data_iter()
            .filter_map(|systems| {
                if systems.server_id == self {
                    Some(SystemId(systems))
                } else {
                    None
                }
            })
            .collect()
    }
}
impl std::ops::Deref for ServerId {
    type Target = ServerData;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl TryFrom<u32> for ServerId {
    type Error = TryFromServerDataIdError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::data()
            .get(value as usize)
            .map(Self)
            .ok_or(TryFromServerDataIdError(value))
    }
}
impl From<ServerId> for u32 {
    fn from(value: ServerId) -> Self {
        value.idx as u32
    }
}
impl std::fmt::Debug for ServerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.idx.fmt(f)
    }
}
impl std::hash::Hash for ServerId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state);
    }
}
impl PartialEq for ServerId {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl Eq for ServerId {}
impl<T> Index<ServerId> for Vec<T> {
    type Output = T;

    fn index(&self, index: ServerId) -> &Self::Output {
        &self[index.idx]
    }
}
impl<T> IndexMut<ServerId> for Vec<T> {
    fn index_mut(&mut self, index: ServerId) -> &mut Self::Output {
        &mut self[index.idx]
    }
}
impl<T> Index<ServerId> for [T] {
    type Output = T;

    fn index(&self, index: ServerId) -> &Self::Output {
        &self[index.idx]
    }
}
impl<T> IndexMut<ServerId> for [T] {
    fn index_mut(&mut self, index: ServerId) -> &mut Self::Output {
        &mut self[index.idx]
    }
}

#[derive(Debug)]
pub struct TryFromServerDataIdError(pub u32);
impl std::fmt::Display for TryFromServerDataIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid hull data id: {} out of bound", self.0)
    }
}

pub fn load_server_data() {
    let read = std::fs::read("../client/tool/server_data/systems.json").unwrap();
    let json: Vec<ServerDataJson> = serde_json::from_slice(read.as_slice()).unwrap();
    DATA.set(
        json.into_iter()
            .zip(0..)
            .map(|(entity_json, id)| entity_json.parse(id))
            .collect(),
    )
    .ok()
    .unwrap();
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ServerDataJson {
    ws_addr: String,
}
impl ServerDataJson {
    fn parse(self, idx: usize) -> ServerData {
        ServerData {
            idx,
            ws_addr: self.ws_addr,
        }
    }
}
