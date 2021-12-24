use common::{
    connection::Connection, idx::ClientId, packets::*, tcp_loops::*, udp_loops::*, Version, SERVER_PING_PORT,
    SERVER_PORT,
};
use std::{
    net::{Ipv6Addr, SocketAddrV6},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    runtime::Runtime,
};

pub struct ConnectionManager {
    pub connection: Connection,
    pub rt: Runtime,
    /// Receive ping time in seconds every second.
    pub ping_duration_receiver: crossbeam_channel::Receiver<f32>,
}
impl ConnectionManager {
    pub fn connect_to_server(addr: &str) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        // Server uses ipv6.
        let server_address = SocketAddrV6::new(addr.parse().unwrap_or(Ipv6Addr::LOCALHOST), SERVER_PORT, 0, 0);

        // Create tokio runtime.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;

        // Start udp loops.
        let socket = Arc::new(std::net::UdpSocket::bind(SocketAddrV6::new(
            Ipv6Addr::UNSPECIFIED,
            0,
            0,
            0,
        ))?);
        let udp_port = socket.local_addr()?.port();
        socket.set_nonblocking(true)?;
        let socket_clone = socket.clone();
        let (udp_connection_event_sender, udp_connection_event_receiver) = crossbeam_channel::unbounded();
        let (udp_packet_to_send, udp_packet_to_send_receiver) = crossbeam_channel::unbounded();
        std::thread::spawn(move || udp_in_loop(socket_clone, udp_connection_event_receiver));
        std::thread::spawn(move || udp_out_loop(socket, udp_packet_to_send_receiver));

        // Create login packet.
        let login_packet = LoginPacket {
            is_steam: true,
            token: 0,
            client_udp_port: udp_port,
            client_version: Version::CURRENT,
        }
        .serialize();

        // Connect tcp stream.
        let mut stream = rt.block_on(async { tokio::net::TcpStream::connect(server_address).await })?;

        // Send login packet.
        rt.block_on(async { stream.write_all(&login_packet).await })?;

        // Get server response.
        let mut buf = [0u8; LoginResponsePacket::FIXED_SIZE];
        rt.block_on(async { stream.read_exact(&mut buf).await })?;
        let login_response = LoginResponsePacket::deserialize(&buf);
        info!("Received login response from server: {:?}.", login_response);

        // Processs LoginResponsePacket.
        let client_id = match login_response {
            LoginResponsePacket::Accepted { client_id } => client_id,
            _ => {
                error!("Server denied login. Reason {:?}. Aborting login...", login_response);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Server denied login."));
            }
        };

        // Add connection to udp loop.
        let (udp_packet_received_sender, udp_packet_received) = crossbeam_channel::unbounded::<Vec<u8>>();
        if let Err(err) = udp_connection_event_sender.send(UdpConnectionEvent::Connected {
            client_udp_addr: server_address,
            udp_packet_received_sender,
        }) {
            error!(
                "{:?} while sending udp connection event to udp in loop. Terminating login attenpt...",
                err
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Can not send udp connection event.",
            ));
        }

        // Wrap stream into buffers.
        let (r, w) = stream.into_split();
        let buf_read = BufReader::new(r);

        // Start tcp loops.
        let (tcp_packet_to_send, tcp_to_send) = tokio::sync::mpsc::channel(8);
        rt.spawn(tcp_out_loop(tcp_to_send, w, ClientId(0)));
        let (tcp_received, tcp_packet_received) = crossbeam_channel::unbounded();
        rt.spawn(tcp_in_loop(
            tcp_received,
            buf_read,
            ClientId(0),
            udp_connection_event_sender.clone(),
            server_address,
        ));

        // Start ping loop.
        let udp_ping_socket = rt
            .block_on(async { tokio::net::UdpSocket::bind(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)).await })?;
        rt.block_on(async {
            udp_ping_socket
                .connect(SocketAddrV6::new(*server_address.ip(), SERVER_PING_PORT, 0, 0))
                .await
        })?;
        let (ping_duration_sender, ping_duration_receiver) = crossbeam_channel::unbounded();
        rt.spawn(ping_loop(udp_ping_socket, ping_duration_sender));

        info!("Connection manager connected to server.");

        let connection = Connection {
            client_id,
            peer_tcp_addr: server_address,
            peer_udp_addr: server_address,
            udp_packet_received,
            tcp_packet_received,
            udp_packet_to_send,
            tcp_packet_to_send,
        };

        Ok(Self {
            connection,
            rt,
            ping_duration_receiver,
        })
    }
}

async fn ping_loop(udp_socket: tokio::net::UdpSocket, ping_duration_sender: crossbeam_channel::Sender<f32>) {
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
        let last_ping = tokio::time::Instant::now();

        tokio::select! {
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
fn is_err_fatal(err: &std::io::Error) -> bool {
    if err.kind() == std::io::ErrorKind::WouldBlock || err.kind() == std::io::ErrorKind::Interrupted {
        false
    } else {
        debug!("Fatal io error {:?}.", err);
        true
    }
}
