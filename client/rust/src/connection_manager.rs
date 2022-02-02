use common::{idx::ClientId, net::{login_packets::*, packets::Packet}, net::tcp_loops::*, Version, net::*};
use std::net::{Ipv6Addr, SocketAddrV6};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::UdpSocket,
    runtime::Runtime,
    spawn,
    task::JoinHandle,
};

/// Try to connect to the server without blocking the main thread.
///
/// Drop this struct to abort the login attempt.
pub struct ConnectionAttempt {
    pub rt: Runtime,
    result_receiver: crossbeam::channel::Receiver<
        std::io::Result<(
            ClientId,
            tokio::sync::mpsc::Sender<TcpOutboundEvent>,
            crossbeam::channel::Receiver<Vec<u8>>,
        )>,
    >,
    _task_handle: JoinHandle<()>,
}
impl ConnectionAttempt {
    pub fn start_login(addr: &str, token: u64) -> std::io::Result<Self> {
        // Server uses ipv6.
        let ip = match addr.parse::<Ipv6Addr>() {
            Ok(ip) => ip,
            Err(err) => {
                error!("{:?} can not parse server ip address. Trying localhost...", err);
                Ipv6Addr::LOCALHOST
            }
        };
        let server_address = SocketAddrV6::new(ip, SERVER_PORT, 0, 0);

        // Create tokio runtime.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;

        // Create result channel.
        let (result_sender, result_receiver) = crossbeam::channel::bounded(1);

        let task_handle = rt.spawn(login_task(server_address, token, result_sender));

        Ok(Self {
            rt,
            result_receiver,
            _task_handle: task_handle,
        })
    }

    pub fn try_receive_result(self) -> Result<ConnectionManager, Result<ConnectionAttempt, std::io::Error>> {
        match self.result_receiver.try_recv() {
            Ok(result) => match result {
                Ok((client_id, tcp_outbound_event_sender, inbound_receiver)) => Ok(ConnectionManager {
                    rt: self.rt,
                    tcp_outbound_event_sender,
                    inbound_receiver,
                    client_id,
                }),
                Err(err) => Err(Err(err)),
            },
            Err(err) => {
                if err.is_disconnected() {
                    Err(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Result sender dropped.",
                    )))
                } else {
                    Err(Ok(self))
                }
            }
        }
    }
}

pub struct ConnectionManager {
    pub rt: Runtime,
    pub client_id: ClientId,
    /// The client can only send packets to the server through tcp.
    tcp_outbound_event_sender: tokio::sync::mpsc::Sender<TcpOutboundEvent>,
    /// The client can receive packets from the server through tcp and udp.
    inbound_receiver: crossbeam::channel::Receiver<Vec<u8>>,
}
impl ConnectionManager {
    pub fn send(&self, packet: &Packet) -> bool {
        self.tcp_outbound_event_sender.blocking_send(TcpOutboundEvent::PacketEvent(packet.serialize())).is_ok()
    }

    pub fn flush(&self) -> bool {
        self.tcp_outbound_event_sender.blocking_send(TcpOutboundEvent::FlushEvent).is_ok()
    }

    pub fn try_recv(&self) -> Result<Vec<u8>, crossbeam::channel::TryRecvError> {
        self.inbound_receiver.try_recv()
    }
}

async fn login_task(
    server_address: SocketAddrV6,
    token: u64,
    result_sender: crossbeam::channel::Sender<
        std::io::Result<(
            ClientId,
            tokio::sync::mpsc::Sender<TcpOutboundEvent>,
            crossbeam::channel::Receiver<Vec<u8>>,
        )>,
    >,
) {
    let result = login(server_address, token).await;
    let _ = result_sender.send(result);
}

async fn login(
    server_address: SocketAddrV6,
    token: u64,
) -> std::io::Result<(
    ClientId,
    tokio::sync::mpsc::Sender<TcpOutboundEvent>,
    crossbeam::channel::Receiver<Vec<u8>>,
)> {
    // Create login packet.
    let login_packet = LoginPacket {
        credential_checker: CredentialChecker::Steam,
        token,
        client_version: Version::CURRENT,
    }
    .serialize();

    // Connect tcp stream.
    let mut stream = tokio::net::TcpStream::connect(server_address).await?;
    stream.set_nodelay(true)?;

    // Send login packet.
    stream.write_all(&login_packet).await?;

    // Get server response.
    let mut buf = [0u8; LoginResponsePacket::FIXED_SIZE];
    stream.read_exact(&mut buf).await?;
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

    // Connect udp socket.
    let socket = UdpSocket::bind(stream.local_addr()?).await?;
    socket.connect(server_address).await?;

    // Wrap stream into buffers.
    let (r, w) = stream.into_split();
    let buf_read = BufReader::new(r);
    let buf_write = BufWriter::new(w);

    // Create channels.
    let (inbound_sender, inbound_receiver) = crossbeam::channel::unbounded();
    let (tcp_outbound_event_sender, tcp_outbound_event_receiver) = tokio::sync::mpsc::channel(32);

    // Start tcp loops.
    spawn(tcp_in_loop(inbound_sender.clone(), buf_read, ClientId(0)));
    spawn(tcp_out_loop(tcp_outbound_event_receiver, buf_write, ClientId(0)));

    // Start udp loop.
    spawn(udp_in_loop(socket, inbound_sender));

    Ok((client_id, tcp_outbound_event_sender, inbound_receiver))
}

async fn udp_in_loop(socket: UdpSocket, inbound_sender: crossbeam::channel::Sender<Vec<u8>>) {
    let mut buf = [0; MAX_UDP_PACKET_SIZE];

    loop {
        match socket.recv(&mut buf).await {
            Ok(num) => {
                if inbound_sender.send(buf[..num].to_vec()).is_err() {
                    debug!("Udp loop disconnected.");
                    break;
                }
            }
            Err(err) => {
                let kind = err.kind();
                if kind != std::io::ErrorKind::Interrupted || kind != std::io::ErrorKind::WouldBlock {
                    error!("{:?} while receiving udp packet. Disconnecting...", err);
                    break;
                }
            }
        }
    }
}
