mod client_inbound;
pub mod client_login;
mod client_outbound;
mod simulation_runner;

use super::*;
use connection::*;
use database::*;
use simulation_runner::*;

struct State {
    database_connection: Connection,

    client_login: AHashMap<client_login::ClientLogin, Connection>,
    client_connections: AHashMap<ClientId, Connection>,

    battlescapes: DashMap<BattlescapeId, BattlescapeRunnerHandle, RandomState>,
}
impl State {}

pub fn _start(db_addr: SocketAddr) {
    // Connect to central server.
    let (central_outbound, mut central_inbound) =
        ConnectionOutbound::connect(CENTRAL_ADDR_INSTANCE).await;
    central_outbound.send(InstanceCentralLoginPacket::new());
    let result = central_inbound
        .recv::<CentralInstanceLoginResult>()
        .await
        .unwrap();
    log::info!("{:?}", result);

    unsafe {
        STATE = Some(State {
            database_outbound: central_outbound,
            battlescapes: Default::default(),
            client_tokens: Default::default(),
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
        handle_central_packet(packet).await;
    }

    log::info!("Instance server stopped");
}

async fn handle_central_packet(packet: CentralInstancePacket) {
    log::debug!("{:?}", packet);
    match packet {
        CentralInstancePacket::ClientChangedBattlescape {
            client_id,
            token,
            battlescape_id,
        } => {
            let old_battlescape_id = if let Some(battlescape_id) = battlescape_id {
                if let Some(battlescape) = state().battlescapes.get(&battlescape_id) {
                    battlescape.clients.lock().insert(client_id);
                }
                state()
                    .client_tokens
                    .insert(client_id, (token, battlescape_id))
                    .map(|v| v.1)
            } else {
                if let Some((_, outbound)) = state().client_connections.remove(&client_id) {
                    outbound.close("Left battlescape");
                }
                state().client_tokens.remove(&client_id).map(|v| v.1 .1)
            };
            if let Some(battlescape_id) = old_battlescape_id {
                if let Some(battlescape) = state().battlescapes.get(&battlescape_id) {
                    battlescape.clients.lock().remove(&client_id);
                }
            }
        }
    }
}

async fn handle_client_packet(packet: ClientInstancePacket, client_id: ClientId) {
    log::debug!("{:?}", packet);
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
        outbound.close("Invalid login packet");
        return;
    };
    log::debug!("{:?}", login);

    // Verify login.
    let Some(battlescape_id) = state()
        .client_tokens
        .get(&login.client_id)
        .and_then(|token| {
            if token.0 == login.token {
                Some(token.1)
            } else {
                None
            }
        })
    else {
        outbound.close("Invalid login token");
        return;
    };

    if let Some(battlescape) = state().battlescapes.get(&battlescape_id) {
        battlescape.clients.lock().insert(login.client_id);
    } else {
        log::warn!("Client connected to non-existent battlescape");
        outbound.close("Battlescape does not exist");
        return;
    }
    state().client_connections.insert(login.client_id, outbound);

    log::debug!("Client login successful");

    // Receive client packets.
    while let Some(packet) = inbound.recv::<ClientInstancePacket>().await {
        handle_client_packet(packet, login.client_id).await;
    }

    // Remove client.
    state().client_connections.remove(&login.client_id);
    // Rest should be removed when central server sends ClientChangedBattlescape.
}

static mut STATE: Option<State> = None;
fn state() -> &'static mut State {
    unsafe { STATE.as_mut().unwrap_unchecked() }
}
