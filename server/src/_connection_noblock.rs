use super::*;
use crate::interval::Interval;
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    thread::spawn,
    time::Duration,
};
use tungstenite::{util::*, Message, WebSocket};

/// Address for the instance servers to connect to.
pub const CENTRAL_ADDR_INSTANCE: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
    std::net::Ipv6Addr::LOCALHOST,
    12461,
    0,
    0,
));
/// Address for the clients to connect to.
pub const CENTRAL_ADDR_CLIENT: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
    std::net::Ipv6Addr::LOCALHOST,
    8461,
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
    pub address: Box<SocketAddr>,

    pub disconnected: bool,

    inbound_receiver: Receiver<Vec<u8>>,
    outbound_sender: Sender<Vec<u8>>,
}
impl Connection {
    pub fn bind(addr: SocketAddr, num_blocking_thread: usize) -> Receiver<Self> {
        let listener = TcpListener::bind(addr).unwrap();
        log::info!("Bound to {}", listener.local_addr().unwrap());

        let (new_internal_connection_sender, new_internal_connection_receiver) =
            channel::<ConnectionInternal>();
        spawn(move || {
            let mut connected: Vec<ConnectionInternal> = Vec::new();

            let mut interval = Interval::new(Duration::from_millis(30));
            loop {
                interval.step();

                match new_internal_connection_receiver.try_recv() {
                    Ok(connection) => connected.push(connection),
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                }

                let mut i = 0;
                while i < connected.len() {
                    let connection = &mut connected[i];

                    if connection.update() {
                        i += 1;
                    } else {
                        connected.swap_remove(i);
                    }
                }
            }
        });

        for _ in 0..num_blocking_thread {
            spawn(|| {
                //
            });
        }

        let (new_connection_sender, new_connection_receiver) = channel();
        spawn(move || {
            while let Ok((stream, address)) = listener.accept() {
                log::debug!("New connection {}", address);
            }

            let mut pending: Vec<WebSocketState> = Vec::new();
            let mut pending_next: Vec<WebSocketState> = Vec::new();

            let mut interval = Interval::new(Duration::from_millis(100));
            'outer: loop {
                interval.step();

                loop {
                    match listener.accept().no_block() {
                        Ok(Some((stream, address))) => {
                            log::debug!("New connection {}", address);

                            if let Some(socket_state) = WebSocketState::new(stream) {
                                pending.push(socket_state);
                            }
                        }
                        Ok(None) => break,
                        Err(err) => {
                            log::warn!("Failed to accept connection: {}", err);
                            break 'outer;
                        }
                    }
                }

                while let Some(socket_state) = pending_next.pop() {
                    log::info!("3");
                    match socket_state.update() {
                        Some(Ok(socket)) => {
                            let Ok(address) = socket.get_ref().peer_addr() else {
                                log::debug!("Failed to get peer address");
                                continue;
                            };

                            let (outbound_sender, outbound_receiver) = channel();
                            let (inbound_sender, inbound_receiver) = channel();

                            let connection_internal = ConnectionInternal {
                                socket,
                                outbound_receiver,
                                inbound_sender,
                            };

                            if new_internal_connection_sender
                                .send(connection_internal)
                                .is_err()
                            {
                                break 'outer;
                            }

                            let connection = Self {
                                address: Box::new(address),
                                disconnected: false,
                                inbound_receiver,
                                outbound_sender,
                            };

                            if new_connection_sender.send(connection).is_err() {
                                break 'outer;
                            }
                        }
                        Some(Err(socket_state)) => {
                            log::info!("2");
                            pending.push(socket_state);
                        }
                        None => {
                            log::info!("1");
                        }
                    }
                }

                std::mem::swap(&mut pending, &mut pending_next);
            }
        });

        new_connection_receiver
    }

    pub fn connect(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(addr).unwrap();

        log::info!("2.1");
        let mut socket = tungstenite::accept(stream).unwrap();
        log::info!("2.2");
        socket.get_mut().set_nodelay(true).unwrap();
        socket.get_mut().set_nonblocking(true).unwrap();

        let (outbound_sender, outbound_receiver) = channel();
        let (inbound_sender, inbound_receiver) = channel();

        let mut connection_internal = ConnectionInternal {
            socket,
            outbound_receiver,
            inbound_sender,
        };

        spawn(move || {
            let mut interval = Interval::new(Duration::from_millis(30));
            while connection_internal.update() {
                interval.step();
            }
        });

        Self {
            address: Box::new(addr),
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

struct ConnectionInternal {
    socket: WebSocket<TcpStream>,
    outbound_receiver: Receiver<Vec<u8>>,
    inbound_sender: Sender<Vec<u8>>,
}
impl ConnectionInternal {
    fn update(&mut self) -> bool {
        loop {
            match self.outbound_receiver.try_recv() {
                Ok(packet) => {
                    if let Err(err) = self.socket.write(Message::Binary(packet)) {
                        // Write should not block.
                        log::debug!("Fatal error while writing message: {}", err);
                        return false;
                    }
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    return false;
                }
            }
        }

        loop {
            match self.socket.read() {
                Ok(msg) => {
                    if let Message::Binary(packet) = msg {
                        if self.inbound_sender.send(packet).is_err() {
                            return false;
                        }
                    }
                }
                Err(err) => {
                    if let Some(err) = err.into_non_blocking() {
                        log::debug!("Fatal error while reading socket: {}", err);
                        return false;
                    } else {
                        break;
                    }
                }
            }
        }

        if let Err(err) = self.socket.flush().no_block() {
            log::debug!("Fatal error while flushing socket: {}", err);
            false
        } else {
            true
        }
    }
}

enum WebSocketState {
    Stream(TcpStream),
    Mid(
        tungstenite::handshake::MidHandshake<
            tungstenite::ServerHandshake<TcpStream, tungstenite::handshake::server::NoCallback>,
        >,
    ),
}
impl WebSocketState {
    fn new(stream: TcpStream) -> Option<Self> {
        if let Err(err) = stream.set_nonblocking(true) {
            log::debug!("Failed to set stream to nonblocking: {}", err);
            return None;
        }

        if let Err(err) = stream.set_nodelay(true) {
            log::debug!("Failed to set stream to nodelay: {}", err);
            return None;
        }

        Some(Self::Stream(stream))
    }

    fn update(self) -> Option<Result<WebSocket<TcpStream>, Self>> {
        match self {
            WebSocketState::Stream(stream) => match tungstenite::accept(stream) {
                Ok(socket) => Some(Ok(socket)),
                Err(tungstenite::HandshakeError::Interrupted(mid)) => Some(Err(Self::Mid(mid))),
                Err(tungstenite::HandshakeError::Failure(err)) => {
                    log::debug!("Failed to accept socket: {}", err);
                    None
                }
            },
            WebSocketState::Mid(mid) => match mid.handshake() {
                Ok(socket) => Some(Ok(socket)),
                Err(tungstenite::HandshakeError::Interrupted(mid)) => Some(Err(Self::Mid(mid))),
                Err(tungstenite::HandshakeError::Failure(err)) => {
                    log::debug!("Failed to accept socket: {}", err);
                    None
                }
            },
        }
    }
}
