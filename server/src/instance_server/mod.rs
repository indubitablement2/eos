use super::*;
use battlescape::*;

const TICK_DURATION: f64 = 1.0 / 24.0;

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

        let mut now = std::time::Instant::now();
        let mut sim_time = 0.0f64;
        let mut real_time = 0.0f64;
        loop {
            real_time += now.elapsed().as_secs_f64();
            now = std::time::Instant::now();

            let dif = sim_time - real_time;
            if dif < -TICK_DURATION * 4.0 {
                log::warn!("Instance server is lagging behind by {} seconds", -dif);
                real_time = sim_time + TICK_DURATION * 4.0;
            } else if dif > 0.001 {
                std::thread::sleep(std::time::Duration::from_secs_f64(dif));
            }

            self.step();
            sim_time += TICK_DURATION;
        }
    }

    fn step(&mut self) {
        //
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
