use super::*;

pub struct Client {
    pub connection: Connection,
}
impl Client {
    // TODO: Add knows data
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ClientView {
    ship: Option<ShipId>,
    translation: Vector2<f32>,
    zoom: f32,
}

#[derive(Deserialize)]
pub enum ClientInbound {
    Test,
}
impl Packet for ClientInbound {
    fn serialize(self) -> Vec<u8> {
        unimplemented!()
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        bincode_decode(&buf)
    }
}

#[derive(Serialize)]
pub enum ClientOutbound {
    ClientShips { ships: Vec<u8> },
}
impl Packet for ClientOutbound {
    fn serialize(self) -> Vec<u8> {
        bincode_encode(self)
    }

    fn parse(_buf: Vec<u8>) -> anyhow::Result<Self> {
        unimplemented!()
    }
}
