mod simulation_runner;

use super::*;
use central_instance::*;
use client_instance::*;
use instance_central::*;
use instance_client::*;
use simulation_runner::SimulationRunnerHandle;

struct State {
    central_outbound: ConnectionOutbound,

    client_auth_request: Mutex<AHashMap<(ClientId, u64), tokio::sync::oneshot::Sender<bool>>>,
    client_connections: RwLock<AHashMap<ClientId, ConnectionOutbound>>,

    simulations: AHashMap<BattlescapeId, SimulationRunnerHandle>,
}

pub async fn _start() {
    log::info!("Starting instance server");

    // Connect to central server.
    let (central_outbound, mut central_inbound) =
        ConnectionOutbound::connect(CENTRAL_ADDR_INSTANCE).await;
    central_outbound.send(InstanceCentralLoginPacket::new());
    let result = central_inbound
        .recv::<CentralInstanceLoginResult>()
        .await
        .unwrap();
    log::debug!("Login result: {:?}", result);

    unsafe {
        STATE = Some(State {
            central_outbound,
            simulations: Default::default(),
            client_auth_request: Default::default(),
            client_connections: Default::default(),
        });
    }

    // Accept client connections.
    let listener = tokio::net::TcpListener::bind(":::0").await.unwrap();
    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_client_connection(stream, addr));
        }
    });

    log::info!("Instance server started");

    // Receive central server packets.
    while let Some(packet) = central_inbound.recv::<CentralInstancePacket>().await {
        handle_instance_packet(packet).await;
    }

    log::info!("Instance server stopped");
}

async fn handle_instance_packet(packet: CentralInstancePacket) {
    match packet {
        CentralInstancePacket::AuthClientResult {
            client_id,
            token,
            success,
        } => {
            if let Some(sender) = state()
                .client_auth_request
                .lock()
                .unwrap()
                .remove(&(client_id, token))
            {
                let _ = sender.send(success);
            }
        }
    }
}

async fn handle_client_packet(packet: ClientInstancePacket, client_id: ClientId) {
    match packet {
        ClientInstancePacket::Test => todo!(),
    }
}

async fn handle_client_connection(stream: tokio::net::TcpStream, addr: SocketAddr) {
    log::debug!("New connection from client {}", addr);

    let Some((outbound, mut inbound)) = ConnectionOutbound::accept(stream).await else {
        return;
    };

    let Some(login) = inbound.recv::<ClientInstanceLoginPacket>().await else {
        return;
    };
    log::debug!("{:?}", login);

    // Verify login.
    let (tx, rx) = tokio::sync::oneshot::channel();
    state()
        .client_auth_request
        .lock()
        .unwrap()
        .insert((login.client_id, login.token), tx);

    state()
        .central_outbound
        .send(InstanceCentralPacket::AuthClient {
            client_id: login.client_id,
            token: login.token,
        });

    if !rx.await.is_ok_and(|success| success) {
        log::debug!("Client auth failed");
        return;
    }
    log::debug!("Client auth success");

    state()
        .client_connections
        .write()
        .unwrap()
        .insert(login.client_id, outbound);

    // Receive client packets.
    while let Some(packet) = inbound.recv::<ClientInstancePacket>().await {
        handle_client_packet(packet, login.client_id).await;
    }

    // Remove client.
    state()
        .client_connections
        .write()
        .unwrap()
        .remove(&login.client_id);
}

static mut STATE: Option<State> = None;
fn state() -> &'static mut State {
    unsafe { STATE.as_mut().unwrap_unchecked() }
}
