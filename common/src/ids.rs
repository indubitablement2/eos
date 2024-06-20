use super::*;

pub trait Id: Sized + Copy {
    fn inner(&mut self) -> &mut InnerId;
    fn next(&mut self) -> Self {
        let ret = *self;
        self.inner().0 = self.inner().0.checked_add(1).unwrap();
        ret
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InnerId(std::num::NonZeroU64);
impl Default for InnerId {
    fn default() -> Self {
        Self(std::num::NonZeroU64::MIN)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct ClientId(InnerId);
impl Id for ClientId {
    fn inner(&mut self) -> &mut InnerId {
        &mut self.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct LeagueId(InnerId);
impl Id for LeagueId {
    fn inner(&mut self) -> &mut InnerId {
        &mut self.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct ClientCharacterId(InnerId);
impl Id for ClientCharacterId {
    fn inner(&mut self) -> &mut InnerId {
        &mut self.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct ServerId(InnerId);
impl Id for ServerId {
    fn inner(&mut self) -> &mut InnerId {
        &mut self.0
    }
}
