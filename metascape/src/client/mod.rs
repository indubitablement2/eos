use super::*;

pub type ClientOwnedFleet = AHashSet<FleetId>;

#[derive(Serialize, Deserialize)]
pub struct Client {
    empty: u8,
    pub auth: Auth,
    pub owned_fleet: ClientOwnedFleet,
    pub faction: FactionId,
}

pub struct ClientBuilder {
    pub auth: Auth,
    pub faction: FactionId,
}
impl ClientBuilder {
    pub fn new(auth: Auth) -> Self {
        Self {
            auth,
            faction: Default::default(),
        }
    }

    pub fn with_faction(mut self, faction: FactionId) -> Self {
        self.faction = faction;
        self
    }

    pub fn build(self) -> Client {
        Client {
            empty: 0,
            auth: self.auth,
            owned_fleet: Default::default(),
            faction: self.faction,
        }
    }
}
