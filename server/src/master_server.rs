use super::*;
use futures_util::{SinkExt, StreamExt};
use parking_lot::{Mutex, RwLock};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{net::TcpStream, spawn};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

type SimulationId = u64;
type SimulationServerId = u64;

pub const MASTER_SERVER_ADDR: &str = "[::1]:6134";
pub const MASTER_PASSWORD: &str = "74K#dL$3p9sv;w;T6%xcp62";

struct Simulation {
    id: SimulationId,
    save: Mutex<Vec<u8>>,
    runner: Mutex<Option<SimulationServerId>>,
}

struct SimulationServer {
    id: SimulationServerId,
    packet_sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    // packet_sender: tokio::sync::<SplitSink<WebSocketStream<TcpStream>, Message>>,
    running_simulations: RwLock<AHashSet<SimulationId>>,
}
impl SimulationServer {
    fn send_simulation(&self, sim_id: SimulationId) {
        if let Some(sim) = MasterServer::get().simulations.read().get(&sim_id) {
            {
                let mut lock = sim.runner.lock();
                if lock.is_some() {
                    error!("simulation is already running");
                    return;
                }
                *lock = Some(self.id);
            }

            self.running_simulations.write().insert(sim_id);

            let _ = self.send_packet(ToSimulationFromMasterPacket::TakeSimulation {
                sim_id,
                sim_data: sim.save.lock().clone(),
            });
        } else {
            error!("simulation {} does not exist", sim_id);
        }
    }

    fn send_packet(&self, packet: ToSimulationFromMasterPacket) -> Result<()> {
        self.packet_sender
            .send(packet.serialize())
            .context("failed to send packet to simulation server")
    }
}

#[derive(Default)]
struct MasterServer {
    next_simulation_id: AtomicU64,
    simulations: RwLock<AHashMap<SimulationId, Arc<Simulation>>>,
    free_simulations: RwLock<Vec<SimulationId>>,

    next_simulation_server_id: AtomicU64,
    simulation_servers: RwLock<AHashMap<SimulationServerId, Arc<SimulationServer>>>,
}
impl MasterServer {
    pub async fn init() {
        unsafe {
            MASTER_SERVER = Some(Self::default());
        }
    }

    pub fn get() -> &'static Self {
        unsafe { MASTER_SERVER.as_ref().unwrap_unchecked() }
    }

    fn add_new_simulation() {
        // TODO
    }

    fn target_simulation_per_server() -> usize {
        let num_server = Self::get().simulation_servers.read().len();
        if num_server == 0 {
            0
        } else {
            (Self::get().simulations.read().len() / num_server) + 1
        }
    }
}
static mut MASTER_SERVER: Option<MasterServer> = None;

pub async fn main() -> Result<()> {
    MasterServer::init().await;

    // Dispense simulations.
    spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(200));
        loop {
            interval.tick().await;

            if MasterServer::get().simulation_servers.read().is_empty() {
                continue;
            }

            let target_num_sim = MasterServer::target_simulation_per_server();

            let mut free_sims: Vec<SimulationId> =
                std::mem::take(MasterServer::get().free_simulations.write().as_mut());

            if free_sims.is_empty() {
                continue;
            }

            for sim_server in MasterServer::get().simulation_servers.read().values() {
                if sim_server.running_simulations.read().len() < target_num_sim {
                    sim_server.send_simulation(free_sims.pop().unwrap());
                    if free_sims.is_empty() {
                        break;
                    }
                }
            }
        }
    });

    // Load balancer.
    spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            if MasterServer::get().simulation_servers.read().is_empty() {
                continue;
            }

            let target_num_sim = MasterServer::target_simulation_per_server();

            for sim_server in MasterServer::get().simulation_servers.read().values() {
                if sim_server.running_simulations.read().len() > target_num_sim {
                    let _ = sim_server
                        .send_packet(ToSimulationFromMasterPacket::RequestReturnSimulation);
                }
            }
        }
    });

    let listener = tokio::net::TcpListener::bind(MASTER_SERVER_ADDR).await?;
    info!("Listening on: {}", listener.local_addr()?);
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(new_connection(stream));
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionRequestPacket {
    SimulationServer {
        password: String,
    },
    Client {
        // TODO: How to id client?
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConnectionResponsePacket {
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToSimulationFromMasterPacket {
    TakeSimulation {
        sim_id: SimulationId,
        sim_data: Vec<u8>,
    },
    /// Return a simulation to the master server if possible.
    RequestReturnSimulation,
}
impl ToSimulationFromMasterPacket {
    fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn deserialize(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToMasterFromSimulationPacket {
    SaveSimulation {
        sim_id: SimulationId,
        save: Vec<u8>,
        return_simulation: bool,
    },
}
impl ToMasterFromSimulationPacket {
    pub fn serialize(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ToClientFromMasterPacket {
    //
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ToMasterFromClientPacket {
    //
}

async fn new_connection(stream: TcpStream) -> Result<()> {
    stream.set_nodelay(false)?;

    let mut stream = tokio_tungstenite::accept_async(stream).await?;

    // TODO: Encryption

    let first: ConnectionRequestPacket = serde_json::from_slice(
        stream
            .next()
            .await
            .ok_or(anyhow!("client disconnected"))??
            .into_data()
            .as_slice(),
    )?;

    match first {
        ConnectionRequestPacket::SimulationServer { password } => {
            if password != MASTER_PASSWORD {
                return Err(anyhow!("invalid password"));
            }

            stream
                .send(Message::Binary(serde_json::to_vec(
                    &ConnectionResponsePacket { success: true },
                )?))
                .await?;

            spawn(sim_server_loop(stream));
        }
        ConnectionRequestPacket::Client {} => {
            // todo
        }
    }

    Ok(())
}

async fn sim_server_loop(stream: WebSocketStream<TcpStream>) {
    let (mut write_ws, mut read_ws) = stream.split();

    let sim_server_id = MasterServer::get()
        .next_simulation_server_id
        .fetch_add(1, Ordering::Relaxed);

    let (packet_sender, mut packet_receiver) = tokio::sync::mpsc::unbounded_channel();

    let sim_server = Arc::new(SimulationServer {
        id: sim_server_id,
        packet_sender,
        running_simulations: RwLock::new(AHashSet::new()),
    });

    MasterServer::get()
        .simulation_servers
        .write()
        .insert(sim_server_id, sim_server.clone());

    // Send packets.
    spawn(async move {
        while let Some(packet) = packet_receiver.recv().await {
            write_ws.send(Message::Binary(packet)).await.unwrap();
        }
    });

    // Receive packet.
    while let Some(Ok(message)) = read_ws.next().await {
        match ToMasterFromSimulationPacket::deserialize(message.into_data().as_slice()) {
            ToMasterFromSimulationPacket::SaveSimulation {
                sim_id,
                save,
                return_simulation,
            } => {
                if let Some(sim) = MasterServer::get().simulations.read().get(&sim_id) {
                    // TODO: Keep backups of old saves.
                    // TODO: Save to file.
                    *sim.save.lock() = save;
                } else {
                    error!("Simulation #{} not found", sim_id);
                }

                if return_simulation {
                    assert!(sim_server.running_simulations.write().remove(&sim_id));

                    MasterServer::get().free_simulations.write().push(sim_id);
                }
            }
        }
    }

    // Remove simulation server from master servers.
    MasterServer::get()
        .simulation_servers
        .write()
        .remove(&sim_server_id);

    // Return all simulations.
    let sim_ids: Vec<SimulationId> = sim_server
        .running_simulations
        .read()
        .iter()
        .copied()
        .collect();
    MasterServer::get().free_simulations.write().extend(sim_ids);

    info!("Simulation server #{} disconnected", sim_server_id);
}
