use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

pub struct Client {
    socket: UdpSocket,
    server_address: SocketAddr,
    send_buffer: Vec<(Vec<u8>, i32)>,
}
impl Client {
    /// Create a default client. "Connected" server is on loopback by default.
    pub fn connect_local() -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)))?;
        let server_address = socket.local_addr()?;

        Ok(Self {
            socket,
            server_address,
            send_buffer: Vec::new(),
        })
    }

    /// Try to connect to a server.
    pub fn connect_server(server_address: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            server_address.port(),
        )))?;
        Ok(Self {
            socket,
            server_address,
            send_buffer: Vec::new(),
        })
    }

    pub fn is_local(&self) -> bool {
        self.server_address.ip().is_loopback()
    }
}
