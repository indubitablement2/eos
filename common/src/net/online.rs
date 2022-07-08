use super::accept_loop::*;
use super::*;
use crate::net::tcp_connection::*;
use crate::net::udp_loops::*;
use ahash::AHashMap;
use std::{
    collections::VecDeque,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket},
};

pub struct OnlineConnection {
    /// The connected peer's udp address.
    udp_addr: SocketAddr,
    tcp_connection: TcpConnection<ClientPacket>,
    udp_inbound_receiver: crossbeam::channel::Receiver<ClientPacket>,
    udp_outbound_sender: crossbeam::channel::Sender<(SocketAddr, Vec<u8>)>,
}
impl OnlineConnection {
    pub fn from_login_success(
        login_success: LoginSuccess,
        udp_inbound_receiver: crossbeam::channel::Receiver<ClientPacket>,
        udp_outbound_sender: crossbeam::channel::Sender<(SocketAddr, Vec<u8>)>,
    ) -> OnlineConnection {
        OnlineConnection {
            udp_addr: login_success.udp_addr,
            tcp_connection: login_success.tcp_connection,
            udp_inbound_receiver,
            udp_outbound_sender,
        }
    }
}
impl Connection for OnlineConnection {
    fn send_reliable(&mut self, packet: &ServerPacket) {
        self.tcp_connection.send_tcp_packet(packet);
    }

    fn send_unreliable(&mut self, packet: &ServerPacket) {
        let _ = self
            .udp_outbound_sender
            .send((self.udp_addr, packets::Packet::serialize(packet)));
    }

    fn recv_packets(&mut self, mut closure: impl FnMut(ClientPacket)) {
        // Recv the udp packets.
        while let Ok(packet) = self.udp_inbound_receiver.try_recv() {
            closure(packet);
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

pub struct OnlineConnectionsManager {
    /// Receive all inbound udp packets.
    /// Packet may not come from a client.
    inbound_udp_receiver: crossbeam::channel::Receiver<(SocketAddr, ClientPacket)>,
    /// Where to send udp packets.
    udp_addr_map: AHashMap<SocketAddr, crossbeam::channel::Sender<ClientPacket>>,

    outbound_udp_v4_sender: Option<crossbeam::channel::Sender<(SocketAddr, Vec<u8>)>>,
    outbound_udp_v6_sender: Option<crossbeam::channel::Sender<(SocketAddr, Vec<u8>)>>,

    pub server_addrs: ServerAddrs,

    login_receiver: crossbeam::channel::Receiver<LoginSuccess>,
    login_queue: VecDeque<LoginSuccess>,

    tick: u32,
    connection_queue_update_interval: u32,
    min_pending_queue_size_for_update: usize,
}
impl OnlineConnectionsManager {
    // TODO: Do that with rayon.
    pub fn update(&mut self) {
        // Receive and sort packets.
        loop {
            match self.inbound_udp_receiver.try_recv() {
                Ok((udp_addr, packet)) => {
                    if let Some(sender) = self.udp_addr_map.get(&udp_addr) {
                        let _ = sender.send(packet);
                    }
                }
                Err(err) => {
                    if err.is_disconnected() {
                        log::error!("Udp loop disconnected. Can not receive inbound udp packets.");
                    }
                    break;
                }
            }
        }

        // Add new login attemps to queue.
        loop {
            match self.login_receiver.try_recv() {
                Ok(login_success) => {
                    self.login_queue.push_back(login_success);
                }
                Err(err) => {
                    if err.is_disconnected() {
                        log::error!("Login loop disconnected. Can not get new login.");
                    }
                    break;
                }
            }
        }

        // Check if we should be updating the connection queue.
        if self.login_queue.len() > self.min_pending_queue_size_for_update
            && self.tick % self.connection_queue_update_interval == 0
        {
            // Check for disconnect while sending queue size.

            let mut disconnected = Vec::new();

            // Send the queue lenght to all queued connection.
            for (connection, len) in self.login_queue.iter_mut().zip(0u32..) {
                if connection
                    .tcp_connection
                    .send_tcp_packet(&ServerPacket::ConnectionQueueLen(ConnectionQueueLen { len }))
                {
                    disconnected.push(len);
                }
            }

            // Remove disconnected login attempt from the queue.
            for i in disconnected.into_iter().rev() {
                self.login_queue.swap_remove_back(i as usize);
            }
        }
    }
}
impl ConnectionsManager for OnlineConnectionsManager {
    type ConnectionType = OnlineConnection;

    fn get_new_login(&mut self, mut closure: impl FnMut(&Auth) -> LoginResponse) -> Option<Self::ConnectionType> {
        if let Some(mut login_success) = self.login_queue.pop_front() {
            // The closure handle converting authenticated connection to client id.
            let mut response = closure(&login_success.auth);

            if let LoginResponse::Accepted { client_id: _ } = &response {
                // Get the matching udp outbound sender.
                let udp_outbound_sender = if login_success.udp_addr.is_ipv4() {
                    self.outbound_udp_v4_sender.as_ref().cloned()
                } else {
                    self.outbound_udp_v6_sender.as_ref().cloned()
                };

                if let Some(udp_outbound_sender) = udp_outbound_sender {
                    let (udp_inbound_sender, udp_inbound_receiver) = crossbeam::channel::unbounded();
                    self.udp_addr_map.insert(login_success.udp_addr, udp_inbound_sender);

                    let mut connection =
                        OnlineConnection::from_login_success(login_success, udp_inbound_receiver, udp_outbound_sender);

                    connection.send_reliable(&ServerPacket::LoginResponse(response));

                    return Some(connection);
                } else {
                    // No sender for ip protocole.
                    response = LoginResponse::NoValidSocket;
                }
            }

            // Notify the connection of why he was denied.
            login_success
                .tcp_connection
                .send_tcp_packet(&ServerPacket::LoginResponse(response));
            login_success.tcp_connection.flush_tcp_buffer();
            None
        } else {
            // No new login.
            None
        }
    }

    fn disconnect(&mut self, connection: Self::ConnectionType) {
        self.udp_addr_map.remove(&connection.udp_addr);
    }

    fn new(configs: &ConnectionConfigs, rt: &tokio::runtime::Runtime) -> Self {
        let port = configs.port;

        let v4 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
        let v6 = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, port, 0, 0));

        let (inbound_udp_sender, inbound_udp_receiver) = crossbeam::channel::unbounded();

        // Start udp loops v4.
        let (outbound_udp_v4_sender, udp_addr_v4) = match UdpSocket::bind(v4) {
            Ok(socket) => {
                let (outbound_udp_v4_sender, outbound_udp_receiver) = crossbeam::channel::unbounded();
                let udp_addr_v4 = socket.local_addr().unwrap();
                let socket_clone = socket.try_clone().unwrap();
                let inbound_udp_sender = inbound_udp_sender.clone();
                std::thread::spawn(move || udp_out_loop(socket, outbound_udp_receiver));
                std::thread::spawn(move || udp_in_loop(socket_clone, inbound_udp_sender));
                (Some(outbound_udp_v4_sender), Some(udp_addr_v4))
            }
            Err(err) => {
                log::warn!("{} creating udp socket for ipv4 address.", err);
                (None, None)
            }
        };

        // Start udp loops v6.
        let (outbound_udp_v6_sender, udp_addr_v6) = match UdpSocket::bind(v6) {
            Ok(socket) => {
                let (outbound_udp_v6_sender, outbound_udp_receiver) = crossbeam::channel::unbounded();
                let udp_addr_v6 = socket.local_addr().unwrap();
                let socket_clone = socket.try_clone().unwrap();
                std::thread::spawn(move || udp_out_loop(socket, outbound_udp_receiver));
                std::thread::spawn(move || udp_in_loop(socket_clone, inbound_udp_sender));
                (Some(outbound_udp_v6_sender), Some(udp_addr_v6))
            }
            Err(err) => {
                log::warn!("{} creating udp socket for ipv6 address.", err);
                (None, None)
            }
        };

        if udp_addr_v4.is_none() && udp_addr_v6.is_none() {
            panic!("could neither bind udp socket with ipv4 or ipv6");
        }

        // Start login loops.
        let (login_sender, login_receiver) = crossbeam::channel::unbounded();
        let v4_addr = if let Some(udp_addr_v4) = udp_addr_v4 {
            let (listener, tcp_addr_v4) = rt.block_on(async move {
                let listener = tokio::net::TcpListener::bind(v4).await.unwrap();
                let addr = listener.local_addr().unwrap();
                (listener, addr)
            });
            let login_sender = login_sender.clone();
            rt.spawn(accept_loop(listener, login_sender));
            Some(ServerAddr {
                udp: udp_addr_v4,
                tcp: tcp_addr_v4,
            })
        } else {
            None
        };
        let v6_addr = if let Some(udp_addr_v6) = udp_addr_v6 {
            let (listener, tcp_addr_v6) = rt.block_on(async move {
                let listener = tokio::net::TcpListener::bind(v6).await.unwrap();
                let addr = listener.local_addr().unwrap();
                (listener, addr)
            });
            let login_sender = login_sender.clone();
            rt.spawn(accept_loop(listener, login_sender));
            Some(ServerAddr {
                udp: udp_addr_v6,
                tcp: tcp_addr_v6,
            })
        } else {
            None
        };

        let server_addrs = ServerAddrs { v4_addr, v6_addr };

        log::info!("Connection manager ready.\n{:#?}", server_addrs,);

        Self {
            login_receiver,
            udp_addr_map: Default::default(),
            server_addrs,
            inbound_udp_receiver,
            outbound_udp_v4_sender,
            outbound_udp_v6_sender,
            login_queue: Default::default(),
            tick: 0,
            connection_queue_update_interval: configs.connection_queue_update_interval,
            min_pending_queue_size_for_update: configs.min_pending_queue_size_for_update,
        }
    }
}
