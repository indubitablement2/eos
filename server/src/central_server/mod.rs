mod client;
mod client_connection;

use self::{client::Client, client_connection::*};
use super::*;
use central_client::*;
use central_instance::*;
use client_central::*;
use instance_central::*;

struct State {
    instances: DashMap<SocketAddr, Instance, RandomState>,
    battlescapes: DashMap<BattlescapeId, Battlescape, RandomState>,

    next_client_id: atomic::AtomicU64,
    clients: DashMap<ClientId, Client, RandomState>,
    username: DashMap<String, ClientId, RandomState>,
    client_connection: DashMap<ClientId, ClientConnection, RandomState>,
}

// TODO: Handle disconnect.
struct Instance {
    connection: ConnectionOutbound,
    battlescapes: Mutex<AHashSet<BattlescapeId>>,
}
impl Instance {
    pub fn send(&self, packet: CentralInstancePacket) {
        self.connection.send(packet);
    }
}

struct Battlescape {
    instance_addr: SocketAddr,
    clients: Mutex<AHashSet<ClientId>>,
}

pub async fn _start() {
    log::info!("Starting central server");

    // TODO: Load state from file.
    unsafe {
        STATE = Some(State {
            instances: Default::default(),
            battlescapes: Default::default(),

            next_client_id: atomic::AtomicU64::new(1),
            clients: Default::default(),
            username: Default::default(),
            client_connection: Default::default(),
        });
    }

    // Instance connection.
    let listener = tokio::net::TcpListener::bind(CENTRAL_ADDR_INSTANCE)
        .await
        .unwrap();
    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_instance_connection(stream, addr));
        }
    });

    // Client connection.
    let listener = tokio::net::TcpListener::bind(CENTRAL_ADDR_CLIENT)
        .await
        .unwrap();
    log::info!("Central server started");
    while let Ok((stream, address)) = listener.accept().await {
        tokio::spawn(handle_client_connection(stream, address));
    }

    log::info!("Central server stopped");
}

async fn handle_instance_packet(packet: InstanceCentralPacket, addr: SocketAddr) {
    log::debug!("{} -> {:?}", addr, packet);
    // match packet {}
}

/// Return and error message if the packet is invalid.
async fn handle_client_packet(
    packet: ClientCentralPacket,
    client_id: ClientId,
) -> Option<&'static str> {
    log::debug!("{:?} -> {:?}", client_id, packet);
    match packet {
        ClientCentralPacket::GlobalMessage { channel, message } => {
            let packet = CentralClientPacket::GlobalMessage {
                from: client_id,
                channel,
                message,
            }
            .serialize();
            // TODO: Only send to client in same channel.
            for connection in state().client_connection.iter() {
                connection.send_raw(packet.clone());
            }
        }
        ClientCentralPacket::JoinBattlescape { new_battlescape_id } => {
            if let Some(battlescape_id) = new_battlescape_id {
                if !state().battlescapes.contains_key(&battlescape_id) {
                    return Some("Battlescape does not exist");
                }
            }

            if let Some(client) = state().client_connection.get(&client_id) {
                client.set_battlescape(new_battlescape_id);
            }
        }
    }

    None
}

async fn handle_client_connection(stream: tokio::net::TcpStream, address: SocketAddr) {
    log::debug!("Client connection attempt: {}", address);

    let Some((outbound, mut inbound)) = ConnectionOutbound::accept(stream).await else {
        return;
    };

    let Some(login) = inbound.recv::<ClientCentralLoginPacket>().await else {
        outbound.close("Invalid login packet");
        return;
    };
    log::debug!("{:?}", login);

    // Verify login.
    let client_id = if login.new_account {
        if let Some((username, password)) = login.username.zip(login.password) {
            if state().username.contains_key(&username) {
                outbound.close("Username already taken");
                return;
            }

            let client_id = ClientId(
                state()
                    .next_client_id
                    .fetch_add(1, atomic::Ordering::Relaxed),
            );

            state().username.insert(username, client_id);

            let client = Client::new(Some(password));
            state().clients.insert(client_id, client);

            client_id
        } else {
            // No other registration method implemented.
            outbound.close("Invalid registration method: Only username+password implemented");
            return;
        }
    } else {
        if let Some((username, password)) = login.username.zip(login.password) {
            let Some(client_id) = state().username.get(&username).as_deref().copied() else {
                outbound.close("Invalid username");
                return;
            };

            if state()
                .clients
                .get(&client_id)
                .is_some_and(|client| client.verify_password(password.as_str()))
            {
                client_id
            } else {
                outbound.close("Invalid password");
                return;
            }
        } else {
            // No other login method implemented.
            outbound.close("Invalid login method: Only username+password implemented");
            return;
        }
    };
    log::debug!("{:?} logged in", client_id);

    let token = rand::random::<u64>();
    outbound.send(CentralClientPacket::LoginSuccess { client_id, token });

    state()
        .client_connection
        .insert(client_id, ClientConnection::new(outbound, client_id, token));

    // Handle packets
    let mut reason = "Unknown error while receiving packets";
    while let Some(packet) = inbound.recv().await {
        if let Some(new_reason) = handle_client_packet(packet, client_id).await {
            reason = new_reason;
            break;
        }
    }

    if let Some((_, connection)) = state().client_connection.remove(&client_id) {
        connection.close(reason);
    }
    log::debug!("{:?} connection fully removed", client_id);
}

async fn handle_instance_connection(stream: tokio::net::TcpStream, addr: SocketAddr) {
    log::debug!("Instance connection attempt: {}", addr);

    let Some((outbound, mut inbound)) = ConnectionOutbound::accept(stream).await else {
        return;
    };

    // Authenticate instance.
    let Some(login) = inbound.recv::<InstanceCentralLoginPacket>().await else {
        return;
    };
    log::debug!("{:?}", login);
    if login.private_key != 123 {
        log::debug!("Invalid private key");
        return;
    }
    let login_result = CentralInstanceLoginResult { nothing: false };
    log::info!("{:?}", login_result);
    outbound.send(login_result);

    state().instances.insert(addr, {
        Instance {
            connection: outbound,
            battlescapes: Default::default(),
        }
    });

    while let Some(packet) = inbound.recv::<InstanceCentralPacket>().await {
        handle_instance_packet(packet, addr).await;
    }

    state().instances.remove(&addr);
    log::warn!("Instance disconnected: {}", addr);
}

static mut STATE: Option<State> = None;
fn state() -> &'static State {
    unsafe { STATE.as_ref().unwrap_unchecked() }
}
