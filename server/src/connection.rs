use super::*;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;

/// Address for the clients to connect to the central server.
pub const CENTRAL_ADDR_CLIENT: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
    std::net::Ipv6Addr::LOCALHOST,
    8461,
    0,
    0,
));
/// Address for the instance servers to connect to the central server.
pub const CENTRAL_ADDR_INSTANCE: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
    std::net::Ipv6Addr::LOCALHOST,
    12461,
    0,
    0,
));
/// Address for the client to connect to the instance servers.
pub const INSTANCE_ADDR_CLIENT: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
    std::net::Ipv6Addr::LOCALHOST,
    7245,
    0,
    0,
));

pub trait SerializePacket {
    fn serialize(self) -> Vec<u8>;
}
pub trait DeserializePacket: Sized {
    fn deserialize(buf: &[u8]) -> Option<Self>;
}

pub struct Connection {
    pub address: SocketAddr,

    pub disconnected: bool,

    inbound_receiver: std::sync::mpsc::Receiver<Vec<u8>>,
    outbound_sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
}
impl Connection {
    pub fn bind_blocking(addr: SocketAddr) -> std::sync::mpsc::Receiver<Self> {
        tokio().block_on(Self::bind(addr))
    }

    pub async fn bind(addr: SocketAddr) -> std::sync::mpsc::Receiver<Self> {
        let (new_connection_sender, new_connection_receiver) = std::sync::mpsc::channel();

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        log::info!("Bound to {}", addr);

        tokio::spawn(async move {
            while let Ok((stream, address)) = listener.accept().await {
                log::debug!("New connection from client {}", address);

                let new_connection_sender = new_connection_sender.clone();
                tokio::spawn(Self::accept_client(stream, address, new_connection_sender));
            }
        });

        new_connection_receiver
    }

    async fn accept_client(
        stream: tokio::net::TcpStream,
        address: SocketAddr,
        new_connection_sender: std::sync::mpsc::Sender<Connection>,
    ) {
        if let Err(err) = stream.set_nodelay(true) {
            log::debug!("Failed to set nodelay on client {}: {}", address, err);
            return;
        }

        let Ok(socket) = tokio_tungstenite::accept_async(stream).await else {
            log::debug!("Failed to accept websocket");
            return;
        };

        let (outbound_sender, mut outbound_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (inbound_sender, inbound_receiver) = std::sync::mpsc::channel();

        let (mut write, mut read) = socket.split();

        tokio::spawn(async move {
            while let Some(packet) = outbound_receiver.recv().await {
                if let Err(err) = write
                    .send(tokio_tungstenite::tungstenite::Message::Binary(packet))
                    .await
                {
                    log::debug!("Failed to send message to client: {}", err);
                    break;
                }
            }
        });
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Binary(packet)) => {
                        if inbound_sender.send(packet).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        log::debug!("Failed to receive message from client: {}", err);
                        break;
                    }
                    _ => {}
                }
            }
        });

        let _ = new_connection_sender.send(Self {
            address,
            disconnected: false,
            inbound_receiver,
            outbound_sender,
        });
    }

    pub fn connect_blocking(addr: SocketAddr) -> Self {
        tokio().block_on(Self::connect(addr))
    }

    pub async fn connect(addr: SocketAddr) -> Self {
        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.set_nodelay(true).unwrap();

        let socket = tokio_tungstenite::client_async(format!("wss://{}", addr), stream)
            .await
            .unwrap()
            .0;

        let (mut write, mut read) = socket.split();

        let (outbound_sender, mut outbound_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        let (inbound_sender, inbound_receiver) = std::sync::mpsc::channel();

        tokio::spawn(async move {
            while let Some(packet) = outbound_receiver.recv().await {
                if let Err(err) = write
                    .send(tokio_tungstenite::tungstenite::Message::Binary(packet))
                    .await
                {
                    log::debug!("Failed to send message to client: {}", err);
                    break;
                }
            }
        });
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Binary(packet)) => {
                        if inbound_sender.send(packet).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        log::debug!("Failed to receive message from client: {}", err);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Self {
            address: addr,
            disconnected: false,
            inbound_receiver,
            outbound_sender,
        }
    }

    pub fn recv<T: DeserializePacket>(&mut self) -> Option<T> {
        match self.inbound_receiver.try_recv() {
            Ok(packet) => {
                let result = T::deserialize(&packet);
                self.disconnected |= result.is_none();
                result
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.disconnected = true;
                None
            }
        }
    }

    pub fn send(&mut self, packet: impl SerializePacket) {
        self.disconnected |= self.outbound_sender.send(packet.serialize()).is_err()
    }
}
