use common::packets::*;
use crossbeam_channel::*;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

pub struct Client {
    local: bool,
    udp_to_send_sender: Sender<UdpClient>,
    udp_received_receiver: Receiver<UdpServer>,

    local_addr: SocketAddrV4,
    peer_addr: SocketAddrV4,
}
impl Client {
    /// Try to connect to a server. This could also be set to loopback if server is also the client.
    pub fn new(server_address: SocketAddrV4) -> Result<Self, std::io::Error> {
        let local_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
        let socket = UdpSocket::bind(local_addr)?;
        socket.connect(server_address)?;
        socket.set_nonblocking(true)?;

        info!(
            "Client connected. Server addr: {:?}. My addr: {:?}.",
            &server_address, &local_addr
        );

        // Create the channels.
        let (udp_to_send_sender, udp_to_send_receiver) = unbounded();
        let (udp_received_sender, udp_received_receiver) = unbounded();

        // Create and start runner.
        ClientRunner::new(socket, udp_received_sender, udp_to_send_receiver).start();

        Ok(Self {
            local: *server_address.ip() == Ipv4Addr::LOCALHOST,
            udp_to_send_sender,
            udp_received_receiver,
            local_addr,
            peer_addr: server_address,
        })
    }

    pub fn get_local_addr(&self) -> SocketAddrV4 {
        self.local_addr
    }

    pub fn get_peer_addr(&self) -> SocketAddrV4 {
        self.peer_addr
    }

    /// If we are connect to loopback address.
    pub fn is_local(&self) -> bool {
        self.local
    }

    /// This will also trigger polling for received packet from the server.
    pub fn send_udp(&self, udp_packet: UdpClient) -> Result<(), SendError<UdpClient>> {
        self.udp_to_send_sender.send(udp_packet)
    }

    /// Need to send to be hable to receive anything.
    pub fn receive_udp(&self) -> Vec<UdpServer> {
        self.udp_received_receiver.try_iter().collect()
    }
}

/// Receive packet from the server.
struct ClientRunner {
    /// Socket connected to the server.
    socket: UdpSocket,
    /// Packet received from the server will be sent to this channel.
    udp_received_sender: Sender<UdpServer>,
    /// Packet to send to the server will be received from this channel.
    udp_to_send_receiver: Receiver<UdpClient>,
    /// Generic buffer used for intermediary socket read.
    datagram_recv_buffer: [u8; 123],
}
impl ClientRunner {
    fn new(socket: UdpSocket, udp_received_sender: Sender<UdpServer>, udp_to_send_receiver: Receiver<UdpClient>) -> Self {
        Self {
            socket,
            udp_received_sender,
            udp_to_send_receiver,
            datagram_recv_buffer: [0u8; 123],
        }
    }

    fn start(mut self) {
        std::thread::spawn(move || {
            info!("ClientReceiver thread started.");

            loop {
                // Send a packet to server.
                match self.udp_to_send_receiver.recv() {
                    Ok(packet) => {
                        // We send without care for any errors that could occur as these packet are dispensable.
                        match self.socket.send(&packet.serialize()) {
                            Ok(num) => {
                                if num != 123 {
                                    warn!("Sent {} bytes to the server while it should've been {}. The server will not be hable to deserialize that.", num, 123);
                                } else {
                                    trace!("Send {} bytes to server.", num);
                                }
                            }
                            Err(err) => trace!("Error while sending udp packet {}.", err),
                        }
                    }
                    Err(_err) => {
                        info!("ClientRunner disconnected.");
                        break;
                    }
                }

                // Receive packet from server.
                while let Ok(num) = self.socket.recv(&mut self.datagram_recv_buffer) {
                    trace!("Got {} bytes from server.", num);

                    // Deserialize buffer.
                    // We don't care about the result. Receiver will disconnect if this ClientRunner is dropped.
                    // let _ = self
                    //     .udp_received_sender
                    //     .send(UdpServer::deserialize(&self.datagram_recv_buffer[..num]));
                }
            }

            info!("ClientReceiver thread finished.");
        });
    }
}
