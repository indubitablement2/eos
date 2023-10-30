use super::*;
use battlescape::*;

const TICK_DURATION: std::time::Duration = std::time::Duration::from_millis(50);

pub struct InstanceServer {
    central_server_connection: Connection,
    // TODO: Client connections
    battlescapes: Option<Battlescape>,
}
impl InstanceServer {
    pub fn start() {
        Self {
            central_server_connection: Connection::connect_blocking(CENTRAL_ADDR_INSTANCE),
            battlescapes: None,
        }
        .run();
    }

    fn run(mut self) {
        log::info!("Instance server started");

        let mut interval = tokio::time::interval(TICK_DURATION);
        loop {
            tokio().block_on(interval.tick());
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum CentralServerPacket {
    //
}
impl SerializePacket for CentralServerPacket {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}
impl DeserializePacket for CentralServerPacket {
    fn deserialize(packet: &[u8]) -> Option<Self> {
        bincode::deserialize(packet).ok()
    }
}

#[derive(Serialize, Deserialize)]
pub enum InstanceServerPacket {
    //
}
impl SerializePacket for InstanceServerPacket {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}
impl DeserializePacket for InstanceServerPacket {
    fn deserialize(packet: &[u8]) -> Option<Self> {
        bincode::deserialize(packet).ok()
    }
}
