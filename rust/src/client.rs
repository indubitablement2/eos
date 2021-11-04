use crossbeam_channel::*;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use common::packets::{UdpClient, UdpServerDeserialized, MAX_DATAGRAM_SIZE};

pub struct Client {
    local: bool,
    udp_to_send_sender: Sender<UdpClient>,
    udp_received_receiver: Receiver<UdpServerDeserialized>,
}
impl Client {
    /// Try to connect to a server. This could also be set to loopback if server is also the client.
    pub fn new(server_address: SocketAddrV4) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
        socket.connect(server_address)?;
        socket.set_nonblocking(true)?;

        info!(
            "Client connected. Server addr: {:?}. My addr: {:?}.",
            &socket.peer_addr()?,
            &socket.local_addr()?
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
        })
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
    pub fn receive_udp(&self) -> Vec<UdpServerDeserialized> {
        self.udp_received_receiver.try_iter().collect()
    }
}

/// Receive packet from the server.
struct ClientRunner {
    /// Socket connected to the server.
    socket: UdpSocket,
    /// Packet received from the server will be sent to this channel.
    udp_received_sender: Sender<UdpServerDeserialized>,
    /// Packet to send to the server will be received from this channel.
    udp_to_send_receiver: Receiver<UdpClient>,
    /// Generic buffer used for intermediary socket read.
    datagram_recv_buffer: [u8; MAX_DATAGRAM_SIZE],
}
impl ClientRunner {
    fn new(
        socket: UdpSocket,
        udp_received_sender: Sender<UdpServerDeserialized>,
        udp_to_send_receiver: Receiver<UdpClient>,
    ) -> Self {
        Self {
            socket,
            udp_received_sender,
            udp_to_send_receiver,
            datagram_recv_buffer: [0u8; MAX_DATAGRAM_SIZE],
        }
    }

    fn start(mut self) {
        std::thread::spawn(move || {
            info!("ClientReceiver thread started.");

            'main: loop {
                // Send packet to server.
                loop {
                    match self.udp_to_send_receiver.try_recv() {
                        Ok(packet) => {
                            // We send without care for any errors that could occur as these packet are dispensable.
                            match self.socket.send(&packet.serialize()) {
                                Ok(num) => {
                                    if num != UdpClient::PAYLOAD_SIZE {
                                        warn!("Sent {} bytes to the server while it should've been {}. The server will not be hable to deserialize that.", num, UdpClient::PAYLOAD_SIZE);
                                    } else {
                                        trace!("Send {} bytes to server.", num);
                                    }
                                }
                                Err(err) => trace!("Error while sending udp packet {}.", err),
                            }
                        }
                        Err(err) => {
                            if err == TryRecvError::Disconnected {
                                info!("ClientRunner disconnected.");
                                break 'main;
                            }
                            break;
                        }
                    }
                }

                // Receive packet from server.
                while let Ok(num) = self.socket.recv(&mut self.datagram_recv_buffer) {
                    trace!("Got {} bytes from server.", num);

                    // Deserialize buffer.
                    match UdpServerDeserialized::deserialize(&self.datagram_recv_buffer[..num]) {
                        Some(packet) => {
                            // We don't care about the result. Receiver will disconnect if this ClientRunner is dropped.
                            let _ = self.udp_received_sender.send(packet);
                        }
                        None => {
                            warn!(
                                "ClientReceiver error while trying to deserialize server packet. Ignoring packet. {:?}",
                                &self.datagram_recv_buffer[..num]
                            );
                        }
                    }
                }
            }

            info!("ClientReceiver thread finished.");
        });
    }
}
