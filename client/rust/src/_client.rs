use common::{idx::ClientId, packets::*, Version};
use crossbeam_channel::Sender;
use std::{
    io::Error,
    net::{Ipv6Addr, SocketAddrV6},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream, UdpSocket,
    },
    runtime::Runtime,
    select,
    time::Instant,
};

pub struct Client {
    pub client_id: ClientId,

    /// Tokio runtime.
    pub rt: Runtime,

    /// Send packet over udp to the server.
    pub udp_sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    /// Receive packet over udp from the server.
    pub udp_receiver: crossbeam_channel::Receiver<Vec<u8>>,
    /// Send packet over tcp to the server.
    pub tcp_sender: tokio::sync::mpsc::UnboundedSender<TcpPacket>,
    /// Receive packet over tcp from the server.
    pub tcp_receiver: crossbeam_channel::Receiver<TcpPacket>,

    pub ping_duration_receiver: crossbeam_channel::Receiver<f32>,

    pub server_addresses: SocketAddrV6,
}
impl Client {
    /// Try to connect to a server. This could also be set to loopback if server is also the client.
    pub fn new(addr: &str) -> std::io::Result<Self> {
        // Server uses ipv6.
        let server_address = SocketAddrV6::new(addr.parse().unwrap_or(Ipv6Addr::LOCALHOST), SERVER_PORT, 0, 0);

        // Create tokio runtime.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;

        // Connect tcp stream.
        let mut tcp_stream = rt.block_on(async { TcpStream::connect(server_addresses.tcp_address).await })?;
        debug!("Connected with server over tcp.");

        // Create UdpSocket.
        let udp_socket =
            Arc::new(rt.block_on(async { UdpSocket::bind(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)).await })?);
        rt.block_on(async { udp_socket.connect(server_addresses.udp_address).await })?;
        debug!("Connected with server over udp.");

        // Create LoginPacket.
        let login_packet = LoginPacket {
            is_steam: true,
            token: 0,
            client_udp_port: udp_socket.local_addr()?.port(),
            client_version: Version::CURRENT,
        }
        .serialize();
        debug!("Created LoginPacket.");

        // Send LoginPacket.
        rt.block_on(async { tcp_stream.write_all(&login_packet).await })?;
        debug!("Sent LoginPacket.");

        // Get server response.
        let mut buf = [0u8; LoginResponsePacket::FIXED_SIZE];
        rt.block_on(async { tcp_stream.read_exact(&mut buf).await })?;
        let login_response = LoginResponsePacket::deserialize(&buf);
        info!("Received login response from server: {:?}.", login_response);

        // Processs LoginResponsePacket.
        let client_id = match login_response {
            LoginResponsePacket::Accepted { client_id } => client_id,
            _ => {
                error!("Server denied login. Reason {:?}. Aborting login...", login_response);
                return Err(Error::new(std::io::ErrorKind::Other, "Server denied login."));
            }
        };

        // Split tcp stream.
        let (r, w) = tcp_stream.into_split();
        let buf_read = BufReader::new(r);
        let buf_write = BufWriter::new(w);

        // Create the channels.
        let (udp_sender, udp_to_send_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (udp_received_sender, udp_receiver) = crossbeam_channel::unbounded();
        let (tcp_sender, tcp_to_send_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (tcp_received_sender, tcp_receiver) = crossbeam_channel::unbounded();

        // start loops.
        let udp_socket_clone = udp_socket.clone();
        rt.spawn(udp_recv_loop(udp_socket_clone, udp_received_sender));
        rt.spawn(udp_send_loop(udp_socket, udp_to_send_receiver));
        rt.spawn(tcp_recv_loop(buf_read, tcp_received_sender));
        rt.spawn(tcp_send_loop(buf_write, tcp_to_send_receiver));

        // Start ping loop.
        let udp_ping_socket =
            rt.block_on(async { UdpSocket::bind(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)).await })?;
        let (ping_duration_sender, ping_duration_receiver) = crossbeam_channel::unbounded();
        rt.spawn(ping_loop(udp_ping_socket, ping_duration_sender));

        Ok(Client {
            client_id,
            rt,
            udp_sender,
            udp_receiver,
            tcp_sender,
            tcp_receiver,
            server_addresses,
            ping_duration_receiver,
        })
    }
}

/// Receive udp from the server.
async fn udp_recv_loop(udp_socket: Arc<UdpSocket>, udp_received_sender: crossbeam_channel::Sender<UdpServer>) {
    let mut recv_buf = [0u8; UdpServer::MAX_SIZE];
    loop {
        match udp_socket.recv(&mut recv_buf).await {
            Ok(num) => match UdpServer::deserialize(&recv_buf[..num]) {
                Some(packet) => {
                    if let Err(err) = udp_received_sender.send(packet) {
                        debug!(
                            "{:?} while sending udp packet on channel. Terminating udp recv loop...",
                            err
                        );
                        break;
                    }
                }
                None => {
                    warn!("Error deserialize udp packet. Ignoring...");
                }
            },
            Err(err) => {
                if is_err_fatal(&err) {
                    error!("Fatal error while reading udp socket. Terminating udp recv loop...");
                    break;
                }
            }
        }
    }
}

/// Send udp to the server.
async fn udp_send_loop(
    udp_socket: Arc<UdpSocket>,
    mut udp_to_send_receiver: tokio::sync::mpsc::UnboundedReceiver<UdpClient>,
) {
    while let Some(packet) = udp_to_send_receiver.recv().await {
        if let Err(err) = udp_socket.send(&packet.serialize()).await {
            if is_err_fatal(&err) {
                error!("Fatal error while writting to udp socket. Terminating udp send loop...");
                break;
            }
        }
    }
    trace!("Udp recv loop finished.");
}

/// Receive tcp from the server.
async fn tcp_recv_loop(
    mut buf_read: BufReader<OwnedReadHalf>,
    tcp_received_sender: crossbeam_channel::Sender<TcpServer>,
) {
    let mut next_payload_size;
    let mut header_buffer = [0u8; 4];
    let mut buf: Vec<u8> = Vec::new();

    loop {
        // Get a header.
        match buf_read.read_exact(&mut header_buffer).await {
            Ok(num) => {
                if num != 4 {
                    warn!("Server disconnected.");
                    break;
                }

                next_payload_size = u32::from_be_bytes(header_buffer) as usize;

                if next_payload_size > buf.len() {
                    // Next packet will need a biger buffer.
                    buf.resize(next_payload_size, 0);
                }

                // Get packet.
                match buf_read.read_exact(&mut buf[..next_payload_size]).await {
                    Ok(num) => {
                        if num != next_payload_size {
                            debug!(
                                "Could not read exactly {} bytes. Got {} instead. Disconnecting...",
                                next_payload_size, num
                            );
                            break;
                        }

                        // Try to deserialize.
                        match TcpServer::deserialize(&buf[..next_payload_size]) {
                            Some(packet) => {
                                // Send packet to channel.
                                if tcp_received_sender.send(packet).is_err() {
                                    debug!("Tcp sender shutdown. Disconnecting...");
                                    break;
                                }
                            }
                            None => {
                                debug!("Error deserializing tcp packet. Disconnecting...");
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        if is_err_fatal(&err) {
                            debug!("Fatal error while reading tcp packet. Disconnecting...");
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                if is_err_fatal(&err) {
                    debug!("Fatal error while reading tcp header. Disconnecting...");
                    break;
                }
            }
        }
    }
}

/// Send tcp to the server.
async fn tcp_send_loop(
    mut buf_write: BufWriter<OwnedWriteHalf>,
    mut tcp_to_send_receiver: tokio::sync::mpsc::UnboundedReceiver<TcpPacket>,
) {
    loop {
        if let Some(packet) = tcp_to_send_receiver.recv().await {
            // Serialize and send data.
            let _ = buf_write.write(&packet.serialize()).await;
            while let Err(err) = buf_write.flush().await {
                if is_err_fatal(&err) {
                    debug!("Fatal error while flushing tcp stream. Disconnecting...");
                    break;
                } else {
                    debug!("Non fatal error while flushing tcp stream. Retrying...");
                }
            }
        } else {
            debug!("Tcp sender shutdown. Disconnecting...");
            break;
        }
    }
}


async fn ping_loop(udp_socket: UdpSocket, ping_duration_sender: Sender<f32>) {
    let mut buf = [0; 1];

    let sleep = tokio::time::sleep(Duration::from_secs(1));
    tokio::pin!(sleep);

    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;

        if let Err(err) = udp_socket.send(&buf).await {
            if is_err_fatal(&err) {
                error!("{} while sending ping to server. Terminating ping loop...", err);
                break;
            }
        }
        let last_ping = Instant::now();

        select! {
            r = udp_socket.recv(&mut buf) => {
                if let Err(err) =  r {
                    if is_err_fatal(&err) {
                        error!("{} while receiving ping. Terminating ping loop...", err);
                        break;
                    } else {
                        continue;
                    }
                }
            }
            _ = &mut sleep => {}
        };

        let ping_time = last_ping.elapsed().as_secs_f32();
        if ping_duration_sender.send(ping_time).is_err() {
            debug!("Ping loop terminated.");
            break;
        }
    }
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