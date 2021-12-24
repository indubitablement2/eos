use common::{
    connection::Connection, idx::ClientId, packets::*, tcp_loops::*, udp_loops::*, Version, SERVER_PING_PORT,
    SERVER_PORT,
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
    pub new_connection_receiver: crossbeam_channel::Receiver<Connection>,
    pub rt: Runtime,
}
impl ConnectionsManager {
    pub fn new(local: bool) -> Result<Self> {
        // Server uses ipv6.
        let addr = match local {
            true => SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, SERVER_PORT, 0, 0),
            false => SocketAddrV6::new(Ipv6Addr::LOCALHOST, SERVER_PORT, 0, 0),
        };

        // Start ping loop.
        let ping_socket = UdpSocket::bind(SocketAddrV6::new(*addr.ip(), SERVER_PING_PORT, 0, 0))?;
        std::thread::spawn(move || ping_loop(ping_socket));

        // Start udp loops.
        let socket = Arc::new(UdpSocket::bind(addr)?);
        socket.set_nonblocking(true)?;
        let socket_clone = socket.clone();
        let (udp_connection_event_sender, udp_connection_event_receiver) = crossbeam_channel::unbounded();
        let (udp_packet_to_send_sender, udp_packet_to_send_receiver) = crossbeam_channel::unbounded();
        std::thread::spawn(move || udp_in_loop(socket_clone, udp_connection_event_receiver));
        std::thread::spawn(move || udp_out_loop(socket, udp_packet_to_send_receiver));

        // Create tokio runtime.
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;

        // Start login loop.
        let listener = rt.block_on(async { TcpListener::bind(addr).await })?;
        let (new_connection_sender, new_connection_receiver) = crossbeam_channel::unbounded();
        rt.spawn(login_loop(
            local,
            listener,
            new_connection_sender,
            udp_connection_event_sender,
            udp_packet_to_send_sender,
        ));

        info!(
            "Connection manager ready.\nAccepting login on {}.\nAccepting ping on {}.",
            SERVER_PORT, SERVER_PING_PORT
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
    client_tcp_addr: SocketAddrV6,
    client_udp_addr: SocketAddrV6,
}

/// Entry point for client.
pub async fn login_loop(
    local: bool,
    listener: TcpListener,
    new_connection_sender: crossbeam_channel::Sender<Connection>,
    udp_connection_event_sender: crossbeam_channel::Sender<UdpConnectionEvent>,
    udp_packet_to_send_sender: crossbeam_channel::Sender<(SocketAddrV6, Vec<u8>)>,
) {
    let (login_result_sender, login_result_receiver) = channel(32);
    tokio::spawn(handle_successful_login(
        login_result_receiver,
        new_connection_sender,
        udp_connection_event_sender,
        udp_packet_to_send_sender,
    ));

    loop {
        match listener.accept().await {
            Ok((new_stream, generic_client_tcp_addr)) => {
                debug!("{} is attempting to login.", generic_client_tcp_addr);

                // Convert generic address to v6.
                let client_tcp_addr = match generic_client_tcp_addr {
                    std::net::SocketAddr::V4(v4) => {
                        debug!("{:?} attempted to connect with an ipv4 address. Ignoring...", v4);
                        continue;
                    }
                    std::net::SocketAddr::V6(v6) => v6,
                };

                tokio::spawn(try_login(
                    local,
                    new_stream,
                    client_tcp_addr,
                    login_result_sender.clone(),
                ));
            }
            Err(err) => {
                debug!("{:?} while listening for new tcp connection. Ignoring...", err);
            }
        }
    }
}

async fn try_login(
    local: bool,
    mut stream: TcpStream,
    client_tcp_addr: SocketAddrV6,
    login_result_sender: tokio::sync::mpsc::Sender<LoginResult>,
) {
    // Get the first packet.
    let mut first_packet_buffer = [0u8; LoginPacket::FIXED_SIZE];
    if let Err(err) = stream.read_exact(&mut first_packet_buffer).await {
        debug!("{:?} while attempting to login a client. Aborting login...", err);
        return;
    }

    // Identify user.
    let (login_response, client_udp_port) = handle_first_packet(local, first_packet_buffer, client_tcp_addr).await;

    // Send login response.
    if let Err(err) = stream.write_all(&login_response.serialize()).await {
        debug!("{:?} while writing login response to stream. Aborting login...", err);
        return;
    }

    if let LoginResponsePacket::Accepted { client_id } = login_response {
        if let Err(err) = login_result_sender
            .send(LoginResult {
                client_id,
                stream,
                client_tcp_addr,
                client_udp_addr: SocketAddrV6::new(*client_tcp_addr.ip(), client_udp_port, 0, 0),
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
    client_tcp_addr: SocketAddrV6,
) -> (LoginResponsePacket, u16) {
    // Deserialize first packet.
    let login_packet = match LoginPacket::deserialize(&first_packet_buffer) {
        Some(p) => {
            trace!(
                "Received valid LoginPacket from {:?}. Attempting login...",
                client_tcp_addr
            );
            p
        }
        None => {
            debug!(
                "Error while deserializing LoginPacket from {:?}. Aborting login...",
                client_tcp_addr
            );
            return (LoginResponsePacket::DeserializeError, 0);
        }
    };

    // Check client version.
    if login_packet.client_version != Version::CURRENT {
        debug!(
            "{} attempted to login with {} which does not match server. Aborting login...",
            client_tcp_addr, login_packet.client_version
        );
        return (
            LoginResponsePacket::WrongVersion {
                server_version: Version::CURRENT,
            },
            0,
        );
    }

    // Check credential.
    let client_id = match local {
        true => {
            debug!("{} logged-in localy as ClientId(1).", client_tcp_addr);
            ClientId(1)
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
                        client_tcp_addr
                    );
                    return (LoginResponsePacket::NotSteam, 0);
                }
            }
        }
    };

    debug!("{} successfully identified as {:?}.", client_tcp_addr, client_id);
    (
        LoginResponsePacket::Accepted { client_id },
        login_packet.client_udp_port,
    )
}

async fn handle_successful_login(
    mut login_result_receiver: tokio::sync::mpsc::Receiver<LoginResult>,
    new_connection_sender: crossbeam_channel::Sender<Connection>,
    udp_connection_event_sender: crossbeam_channel::Sender<UdpConnectionEvent>,
    udp_packet_to_send_sender: crossbeam_channel::Sender<(SocketAddrV6, Vec<u8>)>,
) {
    loop {
        if let Some(login_result) = login_result_receiver.recv().await {
            let (udp_packet_received_sender, udp_packet_received) = crossbeam_channel::unbounded::<Vec<u8>>();

            // Wrap stream into buffers.
            let (r, w) = login_result.stream.into_split();
            let buf_read = BufReader::new(r);

            // Add connection to udp loop.
            if let Err(err) = udp_connection_event_sender.send(UdpConnectionEvent::Connected {
                client_udp_addr: login_result.client_udp_addr,
                udp_packet_received_sender,
            }) {
                debug!(
                    "{:?} while sending udp connection event to udp in loop. Terminating login success handler task...",
                    err
                );
                break;
            }

            // Start tcp loops.
            let (tcp_sender, tcp_to_send) = tokio::sync::mpsc::channel(8);
            tokio::spawn(tcp_out_loop(tcp_to_send, w, login_result.client_id));
            let (tcp_received, tcp_receiver) = crossbeam_channel::unbounded();
            tokio::spawn(tcp_in_loop(
                tcp_received,
                buf_read,
                login_result.client_id,
                udp_connection_event_sender.clone(),
                login_result.client_udp_addr,
            ));

            if new_connection_sender
                .send(Connection {
                    client_id: login_result.client_id,
                    peer_tcp_addr: login_result.client_tcp_addr,
                    peer_udp_addr: login_result.client_udp_addr,
                    udp_packet_received,
                    tcp_packet_received: tcp_receiver,
                    udp_packet_to_send: udp_packet_to_send_sender.clone(),
                    tcp_packet_to_send: tcp_sender,
                })
                .is_err()
            {
                break;
            }
        } else {
            break;
        }
    }
}

fn ping_loop(socket: UdpSocket) {
    let mut buf = [0; 4];
    loop {
        if let Ok((num, ping_addr)) = socket.recv_from(&mut buf) {
            let _ = socket.send_to(&buf[..num], ping_addr);
        }
    }
}
