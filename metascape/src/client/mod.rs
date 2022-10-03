use super::*;

pub type ClientOwnedFleet = Vec<FleetId>;

#[derive(Serialize, Deserialize)]
pub struct Client {
    empty: u8,
    pub auth: Auth,
    pub owned_fleet: ClientOwnedFleet,
}

pub struct ClientBuilder {
    pub auth: Auth,
}
impl ClientBuilder {
    pub fn new(auth: Auth) -> Self {
        Self { auth }
    }

    pub fn build(self) -> Client {
        Client {
            empty: 0,
            auth: self.auth,
            owned_fleet: Default::default(),
        }
    }
}
