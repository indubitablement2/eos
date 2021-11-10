use common::{connection_manager::ServerAddresses, packets::*};
use crossbeam_channel::*;
use std::{
    io::{Read, Write},
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpStream, UdpSocket},
    thread::spawn,
    time::Duration,
};

pub struct Client {
    local: bool,

    pub udp_sender: Sender<UdpClient>,
    pub udp_receiver: Receiver<UdpServer>,
    pub tcp_sender: Sender<TcpClient>,
    pub tcp_receiver: Receiver<TcpServer>,

    local_tcp_address: SocketAddr,
    local_udp_address: SocketAddr,
    server_addresses: ServerAddresses,
}
impl Client {
    /// Try to connect to a server. This could also be set to loopback if server is also the client.
    pub fn new(server_addresses: ServerAddresses) -> std::io::Result<Self> {
        let local = server_addresses.tcp_address.ip().is_loopback();

        // TODO: Use v6, but fall back to v4.
        let addr = match local {
            true => SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0),
            false => SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0),
        };

        // Connect tcp stream.
        let mut tcp_stream = TcpStream::connect(server_addresses.tcp_address)?;
        info!("Connected with server over tcp.");

        // Create UdpSocket.
        let udp_socket = UdpSocket::bind(addr)?;
        udp_socket.connect(server_addresses.udp_address)?;
        info!("Connected with server over udp.");

        // Create LoginPacket.
        let login_packet = match local {
            true => LoginPacket {
                is_steam: false,
                token: 0,
                udp_address: udp_socket.local_addr()?,
            },
            false => {
                // TODO: Get a token from steam.
                todo!()
            }
        }
        .serialize();
        info!("Created LoginPacket.");

        // Set temporary timeouts.
        tcp_stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        tcp_stream.set_write_timeout(Some(Duration::from_secs(10)))?;
        info!("Successfully set temporary read/write timeout on tcp stream.");

        // Send LoginPacket.
        tcp_stream.write_all(&login_packet)?;
        info!("Sent LoginPacket.");

        // Get server response.
        let mut buf = [0u8; LoginResponsePacket::FIXED_SIZE];
        tcp_stream.read_exact(&mut buf)?;
        let login_response = LoginResponsePacket::deserialize(&buf);
        info!("Received login Response: {:?}.", login_response);

        // Processs LoginResponsePacket.
        if login_response != LoginResponsePacket::Accepted {
            error!("Server denied login. Reason {:?}. Aborting login...", login_response);
        }

        // Set timeouts.
        tcp_stream.set_read_timeout(Some(Duration::from_millis(2)))?;
        tcp_stream.set_write_timeout(Some(Duration::from_millis(2)))?;
        udp_socket.set_read_timeout(Some(Duration::from_millis(2)))?;
        udp_socket.set_write_timeout(Some(Duration::from_millis(2)))?;
        info!("Successfully set read/write timeout on sockets.");

        // Create the channels.
        let (udp_sender, udp_to_send_receiver) = unbounded();
        let (udp_received_sender, udp_receiver) = unbounded();
        let (tcp_sender, tcp_to_send_receiver) = unbounded();
        let (tcp_received_sender, tcp_receiver) = unbounded();

        // Create client.
        let client = Client {
            local: tcp_stream.local_addr()?.ip().is_loopback(),
            udp_sender,
            udp_receiver,
            tcp_sender,
            tcp_receiver,
            local_tcp_address: tcp_stream.local_addr()?,
            local_udp_address: udp_socket.local_addr()?,
            server_addresses,
        };

        // Create and start runners.
        spawn(move || udp_loop(udp_socket, udp_to_send_receiver, udp_received_sender));
        spawn(move || tcp_loop(tcp_stream, tcp_to_send_receiver, tcp_received_sender));

        Ok(client)
    }

    /// Get the client's local tcp address.
    pub fn local_tcp_address(&self) -> SocketAddr {
        self.local_tcp_address
    }

    /// Get the client's local udp address.
    pub fn local_udp_address(&self) -> SocketAddr {
        self.local_udp_address
    }

    /// Get the server addresses this client is connected to.
    pub fn server_addresses(&self) -> ServerAddresses {
        self.server_addresses
    }

    /// Return if this client is connected on localhost.
    pub fn is_local(&self) -> bool {
        self.local
    }
}

fn udp_loop(udp_socket: UdpSocket, udp_to_send_receiver: Receiver<UdpClient>, udp_received_sender: Sender<UdpServer>) {
    let mut recv_buf = [0u8; UdpServer::FIXED_SIZE];
    
    info!("Udp loop started.");

    'main: loop {
        // Send to server.
        loop {
            match udp_to_send_receiver.try_recv() {
                Ok(packet) => {
                    if let Err(err) = udp_socket.send(&packet.serialize()) {
                        trace!("{} while sending udp packet. Ignoring...", err);
                    }
                }
                Err(err) => {
                    match err {
                        TryRecvError::Empty => {
                            break;
                        }
                        TryRecvError::Disconnected => {
                            info!("Udp sender channel dropped. Terminating udp loop...");
                            break 'main;
                        }
                    }
                }
            }
        }

        // Receive from server.
        match udp_socket.recv(&mut recv_buf) {
            Ok(num) => {
                // Check number of bytes.
                if num != UdpServer::FIXED_SIZE {
                    warn!("Received an udp packet from server with missing bytes. Ignoring...");
                    continue;
                }

                // Deserialize packet.
                if let Ok(packet) = UdpServer::deserialize(&recv_buf) {
                    if udp_received_sender.send(packet).is_err() {
                        info!("Udp receiver channel dropped. Terminating udp loop...");
                        break 'main;
                    }
                } else {
                    warn!("Received an udp packet from server that could not be deserialized. Ignoring...");
                }
            }
            Err(err) => {
                warn!("{:?} while receiving udp packet from server. Ignoring...", err);
            }
        }
    }

    info!("Udp loop finished.");
}

fn tcp_loop(tcp_stream: TcpStream, tcp_to_send_receiver: Receiver<TcpClient>, tcp_received_sender: Sender<TcpServer>) {
    info!("Tcp loop started.");
    info!("Tcp loop finished.");
}