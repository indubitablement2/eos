use super::*;
use battlescape::*;

pub struct InstanceServer {
    central_server_connection: Connection<CentralServerPacket, InstanceServerPacket>,
    // TODO: Client connections
    battlescapes: Option<Battlescape>,
}
impl InstanceServer {
    pub fn start() {
        Self {
            central_server_connection: tokio().block_on(Connection::connect_instance_to_central()),
            battlescapes: None,
        }
        .run();
    }

    fn run(mut self) {
        log::info!("Instance server started");
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
