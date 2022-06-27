use super::*;

#[derive(Soa, Serialize, Deserialize)]
pub struct Client {
    empty: u8,
}

pub struct ClientBuilder {
}
impl ClientBuilder {
    pub fn new() -> Self {
        Self { }
    }

    pub fn build(self) -> Client {
        Client {
            empty: 0,
        }
    }
}
