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

#[derive(Serialize)]
pub enum ClientOutbound {
    EnteredSystem {
        client_id: ClientId,
        system_id: SimulationId,
    },
    State {
        tick: u64,
        // entities: Vec<EntitySave>,
        // objects: Vec<ObjectSave>,
        // clients: Vec<(ClientId, ClientView)>,
    },
    ClientShips {
        ships: Vec<u8>,
    },
}
impl Packet for ClientOutbound {
    fn serialize(self) -> Vec<u8> {
        bin_encode(self)
    }

    fn parse(_buf: Vec<u8>) -> anyhow::Result<Self> {
        unimplemented!()
    }
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
        bin_decode(&buf)
    }
}
