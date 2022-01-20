use common::{
    connection::Connection, idx::ClientId, packets::*, tcp_loops::*, Version, SERVER_PORT,
};
use std::io::Result;
use std::{
    net::{Ipv6Addr, SocketAddrV6, UdpSocket},
    sync::Arc,
};
use tokio::{
    io::*,
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    sync::mpsc::channel,
};

pub struct ConnectionsManager {
    pub new_connection_receiver: crossbeam::channel::Receiver<Connection>,
    pub rt: Runtime,
}
impl ConnectionsManager {
    pub fn new(local: bool) -> Result<Self> {
        // Server uses ipv6.
        let addr = match local {
            true => SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, SERVER_PORT, 0, 0),
            false => SocketAddrV6::new(Ipv6Addr::LOCALHOST, SERVER_PORT, 0, 0),
        };

        // Start udp loops.
        let socket = Arc::new(UdpSocket::bind(addr)?);
        socket.set_nonblocking(true)?;

        // Create tokio runtime.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        // Start login loop.
        let listener = rt.block_on(async { TcpListener::bind(addr).await })?;
        let (new_connection_sender, new_connection_receiver) = crossbeam::channel::unbounded();
        rt.spawn(login_loop(local, listener, new_connection_sender, socket));

        info!(
            "Connection manager ready. Accepting login on {}.",
            SERVER_PORT
        );

        Ok(Self {
            new_connection_receiver,
            rt,
        })
    }
}

#[derive(Debug)]
struct LoginResult {
    client_id: ClientId,
    stream: TcpStream,
    client_addr: SocketAddrV6,
}

/// Entry point for client.
pub async fn login_loop(
    local: bool,
    listener: TcpListener,
    new_connection_sender: crossbeam::channel::Sender<Connection>,
    socket: Arc<UdpSocket>,
) {
    let (login_result_sender, login_result_receiver) = channel(32);
    tokio::spawn(handle_successful_login(
        login_result_receiver,
        new_connection_sender,
        socket,
    ));

    loop {
        match listener.accept().await {
            Ok((new_stream, generic_client_addr)) => {
                debug!("{} is attempting to login.", generic_client_addr);

                // Convert generic address to v6.
                let client_addr = match generic_client_addr {
                    std::net::SocketAddr::V4(v4) => {
                        debug!(
                            "{:?} attempted to connect with an ipv4 address. Ignoring...",
                            v4
                        );
                        continue;
                    }
                    std::net::SocketAddr::V6(v6) => v6,
                };

                if let Err(err) = new_stream.set_nodelay(true) {
                    debug!("{:?} while setting stream nodelay. Aborting login...", err);
                    continue;
                }

                tokio::spawn(try_login(
                    local,
                    new_stream,
                    client_addr,
                    login_result_sender.clone(),
                ));
            }
            Err(err) => {
                debug!(
                    "{:?} while listening for new tcp connection. Ignoring...",
                    err
                );
            }
        }
    }
}

async fn try_login(
    local: bool,
    mut stream: TcpStream,
    client_addr: SocketAddrV6,
    login_result_sender: tokio::sync::mpsc::Sender<LoginResult>,
) {
    // Get the first packet.
    let mut first_packet_buffer = [0u8; LoginPacket::FIXED_SIZE];
    if let Err(err) = stream.read_exact(&mut first_packet_buffer).await {
        debug!(
            "{:?} while attempting to login a client. Aborting login...",
            err
        );
        return;
    }

    // Identify user.
    let login_response = handle_first_packet(local, first_packet_buffer, client_addr).await;

    // Send login response.
    if let Err(err) = stream.write_all(&login_response.serialize()).await {
        debug!(
            "{:?} while writing login response to stream. Aborting login...",
            err
        );
        return;
    }

    if let LoginResponsePacket::Accepted { client_id } = login_response {
        if let Err(err) = login_result_sender
            .send(LoginResult {
                client_id,
                stream,
                client_addr,
            })
            .await
        {
            debug!(
                "{:?} while sending login result to successful login handler. Aborting login...",
                err
            );
            return;
        }
    }
}

async fn handle_first_packet(
    local: bool,
    first_packet_buffer: [u8; LoginPacket::FIXED_SIZE],
    client_addr: SocketAddrV6,
) -> LoginResponsePacket {
    // Deserialize first packet.
    let login_packet = match LoginPacket::deserialize(&first_packet_buffer) {
        Some(p) => {
            trace!(
                "Received a valid LoginPacket from {:?}. Attempting login...",
                client_addr
            );
            p
        }
        None => {
            debug!(
                "Error while deserializing LoginPacket from {:?}. Aborting login...",
                client_addr
            );
            return LoginResponsePacket::DeserializeError;
        }
    };

    // Check client version.
    if login_packet.client_version != Version::CURRENT {
        debug!(
            "{} attempted to login with {} which does not match server. Aborting login...",
            client_addr, login_packet.client_version
        );
        return LoginResponsePacket::WrongVersion {
            server_version: Version::CURRENT,
        };
    }

    // Check credential.
    let client_id = match local {
        true => {
            debug!("{} logged-in localy.", client_addr);
            ClientId(login_packet.token as u32)
        }
        false => {
            match login_packet.is_steam {
                true => {
                    // TODO: Check credential with steam.
                    todo!()
                }
                false => {
                    debug!(
                        "{} tried to login without steam which is not emplemented. Ignoring...",
                        client_addr
                    );
                    return LoginResponsePacket::NotSteam;
                }
            }
        }
    };

    debug!(
        "{} successfully identified as {:?}.",
        client_addr, client_id
    );
    LoginResponsePacket::Accepted { client_id }
}

async fn handle_successful_login(
    mut login_result_receiver: tokio::sync::mpsc::Receiver<LoginResult>,
    new_connection_sender: crossbeam::channel::Sender<Connection>,
    socket: Arc<UdpSocket>,
) {
    loop {
        if let Some(login_result) = login_result_receiver.recv().await {
            let (inbound_sender, inbound_receiver) = crossbeam::channel::unbounded::<Vec<u8>>();

            // Wrap stream into buffers.
            let (r, w) = login_result.stream.into_split();
            let buf_read = BufReader::new(r);
            let buf_write = BufWriter::new(w);

            // Start tcp loops.
            let (tcp_outbound_event_sender, tcp_outbound_event_receiver) =
                tokio::sync::mpsc::channel(8);
            tokio::spawn(tcp_out_loop(
                tcp_outbound_event_receiver,
                buf_write,
                login_result.client_id,
            ));
            tokio::spawn(tcp_in_loop(
                inbound_sender,
                buf_read,
                login_result.client_id,
            ));

            if new_connection_sender
                .send(Connection::new(
                    login_result.client_id,
                    login_result.client_addr,
                    inbound_receiver,
                    socket.clone(),
                    tcp_outbound_event_sender,
                ))
                .is_err()
            {
                break;
            }
        } else {
            break;
        }
    }
}
