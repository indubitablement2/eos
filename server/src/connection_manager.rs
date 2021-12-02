use crate::ecs_components::ClientId;
use common::packets::*;
use std::{
    collections::HashMap,
    net::{Ipv6Addr, SocketAddr, SocketAddrV6},
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
    server_addresses: ServerAddresses,
}
impl ConnectionsManager {
    pub fn new() -> Result<Self> {
        // Create tokio runtime.
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
        debug!("Create server tokio runtime.");

        // Use v6 only.
        let addr = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, common::SERVER_PORT, 0, 0);

        // Create TcpListener.
        let tcp_listener = rt.block_on(async { TcpListener::bind(addr).await })?;
        debug!("Created server TcpListener.");

        // Create UdpSocket.
        let udp_socket = Arc::new(rt.block_on(async { UdpSocket::bind(addr).await })?);
        debug!("Created server UdpSocket.");

        // Save addresses.
        let server_addresses = ServerAddresses {
            tcp_address: tcp_listener.local_addr()?,
            udp_address: udp_socket.local_addr()?,
        };

        // Start udp receiver loop.
        let udp_senders = Arc::new(Mutex::new(HashMap::with_capacity(32)));
        rt.spawn(recv_udp(udp_senders.clone(), udp_socket.clone()));

        // Create login channel.
        let (new_connection_sender, new_connection_receiver) = crossbeam_channel::unbounded();

        // Start login loop.
        rt.spawn(login_loop(
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
            server_addresses,
        })
    }

    pub fn get_addresses(&self) -> ServerAddresses {
        self.server_addresses
    }
}

async fn login_loop(
    tcp_listener: TcpListener,
    new_connection_sender: crossbeam_channel::Sender<Connection>,
    udp_socket: Arc<UdpSocket>,
    udp_senders: Arc<Mutex<HashMap<SocketAddr, crossbeam_channel::Sender<UdpClient>>>>,
) {
    loop {
        match tcp_listener.accept().await {
            Ok((new_tcp_stream, tcp_addr)) => {
                debug!("{} is attempting to login.", tcp_addr);
                spawn(first_packet(
                    new_tcp_stream,
                    tcp_addr,
                    new_connection_sender.clone(),
                    udp_socket.clone(),
                    udp_senders.clone(),
                ));
            }
            Err(err) => {
                warn!("{:?} while listening for new tcp connection. Ignoring...", err);
            }
        }
    }
}

async fn first_packet(
    new_tcp_stream: TcpStream,
    tcp_addr: SocketAddr,
    new_connection_sender: crossbeam_channel::Sender<Connection>,
    udp_socket: Arc<UdpSocket>,
    udp_senders: Arc<Mutex<HashMap<SocketAddr, crossbeam_channel::Sender<UdpClient>>>>,
) {
    // Wrap stream into buffers.
    let (r, w) = new_tcp_stream.into_split();
    let mut buf_read = BufReader::new(r);
    let buf_write = BufWriter::new(w);

    // Wait for first packet.
    // TODO: Add timeout duration.
    let mut first_packet_buffer = [0u8; LoginPacket::FIXED_SIZE];
    let mut cursor = 0usize;
    while cursor < LoginPacket::FIXED_SIZE - 1 {
        match buf_read.read(&mut first_packet_buffer[cursor..]).await {
            Ok(num) => {
                if num == 0 {
                    debug!("{} disconnected while attempting to login. Aborting...", tcp_addr);
                    return;
                }
                cursor += num;
                trace!("LoginPacket {}/{}", cursor, LoginPacket::FIXED_SIZE - 1);
            }
            Err(err) => {
                debug!("{:?} while attempting to login. Aborting...", err);
                return;
            }
        }
    }

    match LoginPacket::deserialize(&first_packet_buffer) {
        Some(login_packet) => {
            debug!("Received LoginPacket from {}. Attempting login...", tcp_addr);
            try_login(
                login_packet,
                buf_read,
                buf_write,
                tcp_addr,
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

async fn try_login(
    login_packet: LoginPacket,
    buf_read: BufReader<OwnedReadHalf>,
    mut buf_write: BufWriter<OwnedWriteHalf>,
    tcp_addr: SocketAddr,
    new_connection_sender: crossbeam_channel::Sender<Connection>,
    udp_socket: Arc<UdpSocket>,
    udp_senders: Arc<Mutex<HashMap<SocketAddr, crossbeam_channel::Sender<UdpClient>>>>,
) {
    // TODO: Check credential with steam.
    let client_id = match login_packet.is_steam {
        true => {
            // TODO: Check token.
            error!(
                "{} is trying to login with steam. Verifying credential... ***TODO: use ClientId 1 for now***",
                tcp_addr
            );
            ClientId(1)
        }
        false => {
            debug!(
                "{} tried to login without steam which is not implemented. Aborting login...",
                tcp_addr
            );
            return;
        }
    };

    // Send LoginResponse.
    if let Err(err) = buf_write.write(&LoginResponsePacket::Accepted.serialize()).await {
        warn!(
            "{:?} while trying to write LoginResponsePacket to {}. Aborting login...",
            err, tcp_addr
        );
        return;
    }
    if let Err(err) = buf_write.flush().await {
        warn!(
            "{:?} while trying to flush LoginResponsePacket to {}. Aborting login...",
            err, tcp_addr
        );
        return;
    }

    // Start runners.
    let (udp_sender, udp_to_send) = tokio::sync::mpsc::channel(32);
    spawn(send_udp(udp_to_send, udp_socket, login_packet.udp_address));

    let (udp_received, udp_receiver) = crossbeam_channel::unbounded();
    udp_senders
        .lock()
        .unwrap()
        .insert(login_packet.udp_address, udp_received);

    let (tcp_sender, tcp_to_send) = tokio::sync::mpsc::channel(32);
    spawn(send_tcp(tcp_to_send, buf_write, tcp_addr));

    let (tcp_received, tcp_receiver) = crossbeam_channel::unbounded();
    spawn(recv_tcp(
        tcp_received,
        buf_read,
        tcp_addr,
        udp_senders,
        login_packet.udp_address,
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
    udp_address: SocketAddr,
) {
    loop {
        if let Some(packet) = udp_to_send.recv().await {
            // We don't care about being too correct when sending udp.
            match packet.serialize() {
                Ok(buf) => {
                    let _ = udp_socket.send_to(&buf, udp_address).await;
                }
                Err(err) => {
                    debug!("Could not serialize packet {:?}. Not sending...", err);
                }
            }
        } else {
            debug!("Udp sender for {} shutdown.", udp_address);
            break;
        }
    }
}

/// Receive all udp packets.
/// Does not know if a connection is dropped until the tcp receiver channel is dropped.
async fn recv_udp(
    udp_senders: Arc<Mutex<HashMap<SocketAddr, crossbeam_channel::Sender<UdpClient>>>>,
    udp_socket: Arc<UdpSocket>,
) {
    let mut buf = [0u8; UdpClient::FIXED_SIZE];

    loop {
        match udp_socket.recv_from(&mut buf).await {
            Ok((num, addr)) => {
                // Check number of bytes.
                if num != UdpClient::FIXED_SIZE {
                    trace!("{} sent an udp packet with missing bytes. Ignoring...", addr);
                    continue;
                }

                // Deserialize packet.
                if let Some(packet) = UdpClient::deserialize(&buf) {
                    // Check if we have a channel for this addr.
                    if let Some(sender) = udp_senders.lock().unwrap().get(&addr) {
                        if sender.send(packet).is_err() {
                            debug!("{} 's channel is drop and should've been removed.", addr);
                        }
                    } else {
                        trace!("{} sent an udp packet, but is not connected. Ignoring...", addr);
                    }
                } else {
                    trace!(
                        "{} sent an udp packet that could not be deserialized. Ignoring...",
                        addr
                    );
                }
            }
            Err(err) => {
                warn!("{:?} while receiving udp packet from clients. Ignoring...", err);
            }
        }
    }
}

async fn send_tcp(
    mut tcp_to_send: tokio::sync::mpsc::Receiver<TcpServer>,
    mut buf_write: BufWriter<OwnedWriteHalf>,
    tcp_addr: SocketAddr,
) {
    loop {
        if let Some(packet) = tcp_to_send.recv().await {
            // Serialize and send data.
            let _ = buf_write.write(&packet.serialize()).await;
            if let Err(err) = buf_write.flush().await {
                debug!("{} while flushing {} 's tcp stream. Disconnecting...", err, tcp_addr);
                break;
            }
        } else {
            debug!("Tcp sender for {} shutdown.", tcp_addr);
            break;
        }
    }
}

/// If a connection is dropped, also remove from udp addresses.
async fn recv_tcp(
    tcp_received: crossbeam_channel::Sender<TcpClient>,
    mut buf_read: BufReader<OwnedReadHalf>,
    tcp_addr: SocketAddr,
    udp_senders: Arc<Mutex<HashMap<SocketAddr, crossbeam_channel::Sender<UdpClient>>>>,
    udp_address: SocketAddr,
) {
    let mut next_payload_size;
    let mut header_buffer = [0u8; 4];
    let mut buf: Vec<u8> = Vec::new();

    loop {
        // Get a header.
        match buf_read.read_exact(&mut header_buffer).await {
            Ok(num) => {
                if num == 0 {
                    debug!("{} disconnected.", tcp_addr);
                    break;
                }

                next_payload_size = u32::from_be_bytes(header_buffer) as usize;

                if next_payload_size > TcpClient::MAX_SIZE {
                    // Next packet is too large.
                    debug!(
                        "{} tried to send a packet of {} bytes. Disconnecting...",
                        tcp_addr, next_payload_size
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
                            debug!("{} disconnected while sending a packet. Ignoring packet...", tcp_addr);
                            break;
                        }

                        // Try to deserialize.
                        match TcpClient::deserialize(&buf[..next_payload_size]) {
                            Ok(packet) => {
                                // Send packet to channel.
                                if tcp_received.send(packet).is_err() {
                                    debug!("Tcp sender for {} shutdown.", tcp_addr);
                                    break;
                                }
                            }
                            Err(err) => {
                                debug!(
                                    "{} while deserializing {} 's tcp packet. Disconnecting...",
                                    err, tcp_addr
                                );
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        debug!("{} while reading {} 's tcp packet. Disconnecting...", err, tcp_addr);
                        break;
                    }
                }
            }
            Err(err) => {
                debug!("{} while reading {} 's tcp header. Disconnecting...", err, tcp_addr);
                break;
            }
        }
    }

    // Also remove udp address.
    udp_senders.lock().unwrap().remove(&udp_address);
    debug!(
        "Tcp receiver for {} shutdown. Also removed {} from udp list.",
        tcp_addr, udp_address
    );
}
