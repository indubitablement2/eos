mod client;
mod client_connection;

use self::{client::Client, client_connection::*};
use super::*;
use atomic::AtomicU64;
use central_client::*;
use central_instance::*;
use client_central::*;
use instance_central::*;

struct State {
    instances: RwLock<AHashMap<SocketAddr, Instance>>,

    next_client_id: AtomicU64,
    clients: RwLock<AHashMap<ClientId, Client>>,
    username: RwLock<AHashMap<String, ClientId>>,
    client_connection: RwLock<AHashMap<ClientId, ClientConnection>>,
}

// TODO: Handle disconnect.
struct Instance {
    connection: ConnectionOutbound,
}
impl Instance {
    pub fn send(&self, packet: CentralInstancePacket) {
        self.connection.send(packet);
    }
}

pub async fn _start() {
    log::info!("Starting central server");

    // TODO: Load state from file.
    unsafe {
        STATE = Some(State {
            instances: Default::default(),

            next_client_id: AtomicU64::new(1),
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
    match packet {
        InstanceCentralPacket::AuthClient { client_id, token } => {
            let success = if let Some(connection) =
                state().client_connection.read().unwrap().get(&client_id)
            {
                connection.token == token
            } else {
                false
            };

            if let Some(instance) = state().instances.read().unwrap().get(&addr) {
                instance.send(CentralInstancePacket::AuthClientResult {
                    client_id,
                    token,
                    success,
                });
            }
        }
    }
}

async fn handle_client_packet(packet: ClientCentralPacket, client_id: ClientId) {
    match packet {
        ClientCentralPacket::GlobalMessage { channel, message } => {
            let packet = CentralClientPacket::GlobalMessage {
                from: client_id,
                channel,
                message,
            }
            .serialize();
            // TODO: Only send to client in same channel.
            for connection in state().client_connection.read().unwrap().values() {
                connection.send_raw(packet.clone());
            }
        }
    }
}

async fn handle_client_connection(stream: tokio::net::TcpStream, address: SocketAddr) {
    log::debug!("Client connection attempt: {}", address);

    let Some((outbound, mut inbound)) = ConnectionOutbound::accept(stream).await else {
        return;
    };

    let Some(login) = inbound.recv::<ClientCentralLoginPacket>().await else {
        return;
    };
    log::debug!("{:?}", login);

    // Verify login.
    let client_id = if login.new_account {
        if let Some((username, password)) = login.username.zip(login.password) {
            let mut usernames = state().username.write().unwrap();

            if usernames.contains_key(&username) {
                log::debug!("Username already taken");
                return;
            }

            let client_id = ClientId(
                state()
                    .next_client_id
                    .fetch_add(1, atomic::Ordering::Relaxed),
            );

            usernames.insert(username, client_id);
            drop(usernames);

            let client = Client::new_password(password);
            state().clients.write().unwrap().insert(client_id, client);

            client_id
        } else {
            return;
        }
    } else {
        if let Some((username, password)) = login.username.zip(login.password) {
            let Some(client_id) = state().username.read().unwrap().get(&username).copied() else {
                return;
            };

            if state()
                .clients
                .read()
                .unwrap()
                .get(&client_id)
                .is_some_and(|client| client.verify_password(password.as_str()))
            {
                client_id
            } else {
                return;
            }
        } else {
            // No other login method implemented.
            log::debug!("Invalid login method: Only username+password implemented");
            return;
        }
    };
    log::debug!("Client logged in as: {:?}", client_id);

    let token = rand::random::<u64>();
    outbound.send(CentralClientPacket::LoginSuccess { client_id, token });

    state()
        .client_connection
        .write()
        .unwrap()
        .insert(client_id, ClientConnection::new(outbound, token));

    // Handle packets
    while let Some(packet) = inbound.recv().await {
        handle_client_packet(packet, client_id).await;
    }

    state()
        .client_connection
        .write()
        .unwrap()
        .remove(&client_id);
    log::debug!("{:?} disconnected", client_id);
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
    if login.private_key != PRIVATE_KEY {
        log::debug!("Invalid private key");
        return;
    }
    let login_result = CentralInstanceLoginResult { nothing: false };
    log::info!("{:?}", login_result);
    outbound.send(login_result);

    state().instances.write().unwrap().insert(addr, {
        Instance {
            connection: outbound,
        }
    });

    while let Some(packet) = inbound.recv::<InstanceCentralPacket>().await {
        handle_instance_packet(packet, addr).await;
    }

    state().instances.write().unwrap().remove(&addr);
    log::warn!("Instance disconnected: {}", addr);
}

static mut STATE: Option<State> = None;
fn state() -> &'static mut State {
    unsafe { STATE.as_mut().unwrap_unchecked() }
}
