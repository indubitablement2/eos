use super::tcp_connection::*;
use super::udp_loops::*;
use super::*;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::SocketAddrV6;

pub struct OnlineConnectionClientSide {
    /// The connected peer's udp address.
    udp_addr: SocketAddr,
    tcp_connection: TcpConnection<ServerPacket>,
    inbound_udp_receiver: crossbeam::channel::Receiver<(SocketAddr, ServerPacket)>,
    outbound_udp_sender: crossbeam::channel::Sender<(SocketAddr, Vec<u8>)>,
}
impl OnlineConnectionClientSide {
    /// Try to connect to the requested server.
    ///
    /// Result will be sent to a channel.
    pub fn connect(
        server_addrs: ServerAddrs,
        credential_checker: CredentialChecker,
        rt: &tokio::runtime::Runtime,
    ) -> crossbeam::channel::Receiver<(LoginResponse, Self)> {
        let (result_sender, result_receiver) = crossbeam::channel::unbounded();
        rt.spawn(connect_to_server(server_addrs, credential_checker, result_sender));
        result_receiver
    }
}
impl ConnectionClientSide for OnlineConnectionClientSide {
    fn send_reliable(&mut self, packet: &ClientPacket) {
        self.tcp_connection.send_tcp_packet(packet);
    }

    fn send_unreliable(&mut self, packet: &ClientPacket) {
        let _ = self
            .outbound_udp_sender
            .send((self.udp_addr, packets::Packet::serialize(packet)));
    }

    fn recv_packets(&mut self, mut closure: impl FnMut(ServerPacket)) {
        // Recv the udp packets.
        while let Ok((addr, packet)) = self.inbound_udp_receiver.try_recv() {
            if self.udp_addr == addr {
                closure(packet);
            } else {
                log::debug!("Received an udp packet from an unknow address. Ignoring...");
            }
        }

        // Recv the tcp packets.
        while let Ok(packet) = self.tcp_connection.inbound_tcp_receiver.try_recv() {
            closure(packet);
        }
    }

    fn flush(&mut self) -> bool {
        !self.tcp_connection.flush_tcp_buffer()
    }
}

async fn connect_to_server(
    server_addrs: ServerAddrs,
    credential_checker: CredentialChecker,
    result_sender: crossbeam::channel::Sender<(LoginResponse, OnlineConnectionClientSide)>,
) -> std::io::Result<()> {
    // Try to bind to ip v6.
    let result = if let Some(v6_addr) = &server_addrs.v6_addr {
        std::net::UdpSocket::bind(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)).map(|socket| (socket, v6_addr))
    } else {
        Err(std::io::Error::other("server does not support ip v6"))
    };

    // Fall back to ip v4.
    let (socket, server_addrs) = if let Ok(result) = result {
        result
    } else if let Some(v4_addr) = &server_addrs.v4_addr {
        (
            std::net::UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?,
            v4_addr,
        )
    } else {
        log::error!("Server does not support fall back ip v4.");
        return Ok(());
    };

    // The address we want to receive udp packet to.
    let requested_udp_addr = socket.local_addr()?;

    let login_packet = LoginPacket {
        credential_checker,
        requested_udp_addr,
    };

    // Connect to the server over tcp.
    let mut tcp_connection = TcpConnection::new(tokio::net::TcpStream::connect(server_addrs.tcp).await?).await;
    tcp_connection.send_tcp_packet(&ClientPacket::LoginPacket(login_packet));
    if !tcp_connection.flush_tcp_buffer() {
        log::warn!("Could not send login packet. Aborting login attempt...");
        return Ok(());
    }

    // Create udp loops.
    let (inbound_udp_sender, inbound_udp_receiver) = crossbeam::channel::unbounded();
    let (outbound_udp_sender, outbound_udp_receiver) = crossbeam::channel::unbounded();
    let socket_clone = socket.try_clone()?;
    udp_in_loop(socket_clone, inbound_udp_sender);
    udp_out_loop(socket, outbound_udp_receiver);

    // Wait for connectiong result.
    if let Ok(packet) = tcp_connection.inbound_tcp_receiver.recv() {
        if let ServerPacket::LoginResponse(response) = packet {
            let connection = OnlineConnectionClientSide {
                udp_addr: server_addrs.udp,
                tcp_connection,
                inbound_udp_receiver,
                outbound_udp_sender,
            };

            let _ = result_sender.send((response, connection));
        } else {
            log::error!("First server packet is not a login response.");
        }
    }

    Ok(())
}
