use common::idx::*;
use common::packets::*;
use std::{
    collections::HashMap,
    net::{Ipv6Addr, SocketAddrV6},
    sync::{Arc, Mutex},
};
use tokio::{
    io::*,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream, UdpSocket,
    },
    runtime::Runtime,
    spawn,
};

pub struct Connection {
    pub client_id: ClientId,
    pub udp_sender: tokio::sync::mpsc::Sender<UdpServer>,
    pub udp_receiver: crossbeam_channel::Receiver<UdpClient>,
    pub tcp_sender: tokio::sync::mpsc::Sender<TcpServer>,
    pub tcp_receiver: crossbeam_channel::Receiver<TcpClient>,
}

pub struct ConnectionsManager {
    pub new_connection_receiver: crossbeam_channel::Receiver<Connection>,
    _rt: Runtime,
}
impl ConnectionsManager {
    pub fn new(local: bool) -> Result<Self> {
        // Create tokio runtime.
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
        debug!("Create server tokio runtime.");

        let addr = match local {
            true => SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, common::SERVER_PORT, 0, 0),
            false => SocketAddrV6::new(Ipv6Addr::LOCALHOST, common::SERVER_PORT, 0, 0),
        };

        // Create TcpListener.
        let tcp_listener = rt.block_on(async { TcpListener::bind(addr).await })?;
        debug!("Created server TcpListener.");

        // Create UdpSocket.
        let udp_socket = Arc::new(rt.block_on(async { UdpSocket::bind(addr).await })?);
        debug!("Created server UdpSocket.");

        // Start udp receiver loop.
        let udp_senders = Arc::new(Mutex::new(HashMap::with_capacity(32)));
        rt.spawn(recv_udp(udp_senders.clone(), udp_socket.clone()));

        // Create login channel.
        let (new_connection_sender, new_connection_receiver) = crossbeam_channel::unbounded();

        // Start login loop.
        rt.spawn(login_loop(
            local,
            tcp_listener,
            new_connection_sender,
            udp_socket,
            udp_senders.clone(),
        ));
        debug!("Started login loop.");

        info!("Server ready.");

        Ok(Self {
            new_connection_receiver,
            _rt: rt,
        })
    }
}

/// Entry point for client.
async fn login_loop(
    local: bool,
    tcp_listener: TcpListener,
    new_connection_sender: crossbeam_channel::Sender<Connection>,
    udp_socket: Arc<UdpSocket>,
    udp_senders: Arc<Mutex<HashMap<SocketAddrV6, crossbeam_channel::Sender<UdpClient>>>>,
) {
    loop {
        match tcp_listener.accept().await {
            Ok((new_tcp_stream, generic_client_tcp_addr)) => {
                debug!("{} is attempting to login.", generic_client_tcp_addr);

                // Convert generic address to v6.
                let client_tcp_addr = match generic_client_tcp_addr {
                    std::net::SocketAddr::V4(v4) => {
                        warn!("Client tcp address is v4. Aborting login...");
                        continue;
                    }
                    std::net::SocketAddr::V6(v6) => v6,
                };

                spawn(first_packet(
                    local,
                    new_tcp_stream,
                    client_tcp_addr,
                    new_connection_sender.clone(),
                    udp_socket.clone(),
                    udp_senders.clone(),
                ));
            }
            Err(err) => {
                debug!("{:?} while listening for new tcp connection. Ignoring...", err);
            }
        }
    }
}

/// Get the client's first packet.
async fn first_packet(
    local: bool,
    new_tcp_stream: TcpStream,
    client_tcp_addr: SocketAddrV6,
    new_connection_sender: crossbeam_channel::Sender<Connection>,
    udp_socket: Arc<UdpSocket>,
    udp_senders: Arc<Mutex<HashMap<SocketAddrV6, crossbeam_channel::Sender<UdpClient>>>>,
) {
    // Wrap stream into buffers.
    let (r, write_half) = new_tcp_stream.into_split();
    let mut buf_read = BufReader::new(r);

    // Get the first packet.
    // TODO: Add timeout duration.
    let mut first_packet_buffer = [0u8; LoginPacket::FIXED_SIZE];
    if let Err(err) = buf_read.read_exact(&mut first_packet_buffer).await {
        debug!("{:?} while attempting to login a client. Aborting login...", err);
        return;
    }

    // let mut cursor = 0usize;
    // while cursor < LoginPacket::FIXED_SIZE - 1 {
    //     match buf_read.read(&mut first_packet_buffer[cursor..]).await {
    //         Ok(num) => {
    //             if num == 0 {
    //                 debug!("{} disconnected while attempting to login. Aborting login...", client_tcp_addr);
    //                 return;
    //             }
    //             cursor += num;
    //             trace!("LoginPacket {}/{}", cursor, LoginPacket::FIXED_SIZE - 1);
    //         }
    //         Err(err) => {
    //             debug!("{:?} while attempting to login. Aborting login...", err);
    //             return;
    //         }
    //     }
    // }

    match LoginPacket::deserialize(&first_packet_buffer) {
        Some(login_packet) => {
            debug!("Received LoginPacket from {}. Attempting login...", client_tcp_addr);
            try_login(
                local,
                login_packet,
                buf_read,
                write_half,
                client_tcp_addr,
                new_connection_sender,
                udp_socket,
                udp_senders,
            )
            .await;
        }
        None => {
            debug!("Error while deserializing LoginPacket. Aborting login...");
        }
    }
}

/// Identify the client.
async fn try_login(
    local: bool,
    login_packet: LoginPacket,
    buf_read: BufReader<OwnedReadHalf>,
    mut write_half: OwnedWriteHalf,
    client_tcp_addr: SocketAddrV6,
    new_connection_sender: crossbeam_channel::Sender<Connection>,
    udp_socket: Arc<UdpSocket>,
    udp_senders: Arc<Mutex<HashMap<SocketAddrV6, crossbeam_channel::Sender<UdpClient>>>>,
) {
    let client_id = match local {
        true => {
            debug!("{} logged-in localy as ClientId 1", client_tcp_addr);
            ClientId(1)
        }
        false => {
            match login_packet.is_steam {
                true => {
                    // TODO: Check credential with steam.
                    error!(
                        "{} is trying to login with steam. Verifying credential... ***TODO: use ClientId 1 for now***",
                        client_tcp_addr
                    );
                    ClientId(1)
                }
                false => {
                    debug!(
                        "{} tried to login without steam which is not implemented. Aborting login...",
                        client_tcp_addr
                    );
                    return;
                }
            }
        }
    };

    // Send LoginResponse.
    if let Err(err) = write_half
        .write_all(&LoginResponsePacket::Accepted { client_id }.serialize())
        .await
    {
        warn!(
            "{:?} while trying to write LoginResponsePacket to {}. Aborting login...",
            err, client_tcp_addr
        );
        return;
    }

    let client_udp_address = SocketAddrV6::new(*client_tcp_addr.ip(), login_packet.client_udp_port, 0, 0);

    // Start runners.
    let (udp_sender, udp_to_send) = tokio::sync::mpsc::channel(32);
    spawn(send_udp(udp_to_send, udp_socket, client_udp_address));

    let (udp_received, udp_receiver) = crossbeam_channel::unbounded();
    udp_senders.lock().unwrap().insert(client_udp_address, udp_received);

    let (tcp_sender, tcp_to_send) = tokio::sync::mpsc::channel(32);
    spawn(send_tcp(tcp_to_send, write_half, client_tcp_addr));

    let (tcp_received, tcp_receiver) = crossbeam_channel::unbounded();
    spawn(recv_tcp(
        tcp_received,
        buf_read,
        client_tcp_addr,
        udp_senders,
        client_udp_address,
    ));

    // Create Connection.
    let connection = Connection {
        client_id,
        udp_sender,
        udp_receiver,
        tcp_sender,
        tcp_receiver,
    };

    let _ = new_connection_sender.send(connection);
}

async fn send_udp(
    mut udp_to_send: tokio::sync::mpsc::Receiver<UdpServer>,
    udp_socket: Arc<UdpSocket>,
    client_udp_address: SocketAddrV6,
) {
    loop {
        if let Some(packet) = udp_to_send.recv().await {
            if let Err(err) = udp_socket.send_to(&packet.serialize(), client_udp_address).await {
                if is_err_fatal(&err) {
                    debug!(
                        "Fatal error while sending packet to {}. Disconnecting...",
                        client_udp_address
                    );
                    break;
                }
            }
        } else {
            debug!("Udp sender for {} shutdown.", client_udp_address);
            break;
        }
    }
}

/// Receive all udp packets.
/// Does not know if a connection is dropped until the tcp receiver channel is dropped.
async fn recv_udp(
    udp_senders: Arc<Mutex<HashMap<SocketAddrV6, crossbeam_channel::Sender<UdpClient>>>>,
    udp_socket: Arc<UdpSocket>,
) {
    let mut buf = [0u8; 1200];

    loop {
        match udp_socket.recv_from(&mut buf).await {
            Ok((num, generic_client_udp_addr)) => {
                // Convert generic address to v6.
                let client_udp_addr = match generic_client_udp_addr {
                    std::net::SocketAddr::V4(v4) => {
                        trace!("Got an udp packet from a v4 address. Ignoring...");
                        continue;
                    }
                    std::net::SocketAddr::V6(v6) => v6,
                };

                // Check if we have a channel for this addr.
                if let Some(sender) = udp_senders.lock().unwrap().get(&client_udp_addr) {
                    // Deserialize packet.
                    if let Some(packet) = UdpClient::deserialize(&buf[..num]) {
                        if sender.send(packet).is_err() {
                            warn!(
                                "{} 's channel is drop and should've been removed. Ignoring...",
                                client_udp_addr
                            );
                        }
                    } else {
                        trace!(
                            "{} sent an udp packet that could not be deserialized. Ignoring...",
                            client_udp_addr
                        );
                    }
                } else {
                    trace!(
                        "{} sent an udp packet, but is not connected. Ignoring...",
                        client_udp_addr
                    );
                }
            }
            Err(err) => {
                debug!("{:?} while receiving udp packet from clients. Ignoring...", err);
            }
        }
    }
}

async fn send_tcp(
    mut tcp_to_send: tokio::sync::mpsc::Receiver<TcpServer>,
    mut write_half: OwnedWriteHalf,
    client_tcp_addr: SocketAddrV6,
) {
    loop {
        if let Some(packet) = tcp_to_send.recv().await {
            // Serialize and send data.
            if let Err(err) = write_half.write_all(&packet.serialize()).await {
                if is_err_fatal(&err) {
                    debug!(
                        "Fatal error while writting to {} 's tcp socket. Disconnecting...",
                        client_tcp_addr
                    );
                    break;
                }
            }
        } else {
            debug!("Tcp sender for {} shutdown.", client_tcp_addr);
            break;
        }
    }
}

/// If a connection is dropped, also remove from udp addresses.
async fn recv_tcp(
    tcp_received: crossbeam_channel::Sender<TcpClient>,
    mut buf_read: BufReader<OwnedReadHalf>,
    client_tcp_addr: SocketAddrV6,
    udp_senders: Arc<Mutex<HashMap<SocketAddrV6, crossbeam_channel::Sender<UdpClient>>>>,
    udp_address: SocketAddrV6,
) {
    let mut next_payload_size;
    let mut header_buffer = [0u8; 4];
    let mut buf: Vec<u8> = Vec::new();

    loop {
        // Get a header.
        match buf_read.read_exact(&mut header_buffer).await {
            Ok(num) => {
                if num == 0 {
                    debug!("{} disconnected.", client_tcp_addr);
                    break;
                }

                next_payload_size = u32::from_be_bytes(header_buffer) as usize;

                if next_payload_size > TcpClient::MAX_SIZE {
                    // Next packet is too large.
                    debug!(
                        "{} tried to send a packet of {} bytes which is over size limit of {}. Disconnecting...",
                        client_tcp_addr,
                        next_payload_size,
                        TcpClient::MAX_SIZE
                    );
                    break;
                } else if next_payload_size > buf.len() {
                    // Next packet will need a biger buffer.
                    buf.resize(next_payload_size, 0);
                }

                // Get packet.
                match buf_read.read_exact(&mut buf[..next_payload_size]).await {
                    Ok(num) => {
                        if num != next_payload_size {
                            debug!(
                                "{} disconnected while sending a packet. Ignoring packet...",
                                client_tcp_addr
                            );
                            break;
                        }

                        // Try to deserialize.
                        match TcpClient::deserialize(&buf[..next_payload_size]) {
                            Some(packet) => {
                                // Send packet to channel.
                                if tcp_received.send(packet).is_err() {
                                    debug!("Tcp sender for {} shutdown.", client_tcp_addr);
                                    break;
                                }
                            }
                            None => {
                                debug!(
                                    "Error while deserializing {} 's tcp packet. Disconnecting...",
                                    client_tcp_addr
                                );
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        if is_err_fatal(&err) {
                            debug!(
                                "Fatal error while reading {} 's tcp packet. Disconnecting...",
                                client_tcp_addr
                            );
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                if is_err_fatal(&err) {
                    debug!(
                        "Fatal error while reading {} 's tcp header. Disconnecting...",
                        client_tcp_addr
                    );
                    break;
                }
            }
        }
    }

    // Also remove udp address.
    udp_senders.lock().unwrap().remove(&udp_address);
    debug!(
        "Tcp receiver for {} shutdown. Also removed {} from udp list.",
        client_tcp_addr, udp_address
    );
}

/// If the io error is fatal, return true and print the err to debug log.
fn is_err_fatal(err: &Error) -> bool {
    if err.kind() == std::io::ErrorKind::WouldBlock || err.kind() == std::io::ErrorKind::Interrupted {
        false
    } else {
        debug!("Fatal io error {:?}.", err);
        true
    }
}
