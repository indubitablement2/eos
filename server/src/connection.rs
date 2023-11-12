use super::*;
use std::net::TcpListener;
use std::net::TcpStream;
use tungstenite::util::*;
use tungstenite::{HandshakeError, Message, WebSocket};

pub trait Packet: Sized {
    fn serialize(self) -> Vec<u8>;
    fn parse(buf: Vec<u8>) -> Result<Self, &'static str>;
}

pub struct ConnectionListener {
    listener: TcpListener,
    accept_ongoing: Vec<
        tungstenite::handshake::MidHandshake<
            tungstenite::ServerHandshake<TcpStream, tungstenite::handshake::server::NoCallback>,
        >,
    >,
    login_ongoing: Vec<Connection>,
    login_failure: Vec<Connection>,
}
impl ConnectionListener {
    pub fn bind(addr: SocketAddr) -> Self {
        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).unwrap();

        Self {
            listener,
            accept_ongoing: Vec::new(),
            login_ongoing: Vec::new(),
            login_failure: Vec::new(),
        }
    }

    pub fn accept<T: Packet>(
        &mut self,
        mut login: impl FnMut(T) -> Result<(), &'static str>,
        mut new_connection: impl FnMut(Connection),
    ) {
        let mut accept = Vec::new();

        loop {
            match self.listener.accept().no_block() {
                Ok(Some((stream, _))) => {
                    if let Err(err) = stream.set_nodelay(true) {
                        log::warn!("Failed to set nodelay: {}", err);
                    }

                    if let Err(err) = stream.set_nonblocking(true) {
                        log::warn!("Failed to set non-blocking: {}", err);
                        continue;
                    }

                    match tungstenite::accept(stream) {
                        Ok(socket) => {
                            self.login_ongoing.push(Connection { socket });
                        }
                        Err(HandshakeError::Interrupted(mid)) => {
                            self.accept_ongoing.push(mid);
                        }
                        Err(HandshakeError::Failure(err)) => {
                            log::debug!("Failed to accept connection: {}", err);
                        }
                    }
                }
                Ok(None) => break,
                Err(err) => {
                    log::error!("Failed to accept connection: {}", err);
                    break;
                }
            }
        }

        std::mem::swap(&mut self.accept_ongoing, &mut accept);
        for mid in accept {
            match mid.handshake() {
                Ok(socket) => {
                    self.login_ongoing.push(Connection { socket });
                }
                Err(HandshakeError::Interrupted(mid)) => self.accept_ongoing.push(mid),
                Err(HandshakeError::Failure(err)) => {
                    log::debug!("Failed to accept connection: {}", err);
                }
            }
        }

        let mut i = 0;
        while i < self.login_ongoing.len() {
            let connection = &mut self.login_ongoing[i];

            if connection.is_closed() {
                self.login_ongoing.swap_remove(i);
                continue;
            }

            if let Some(t) = connection.recv() {
                let mut connection = self.login_ongoing.swap_remove(i);

                if let Err(reason) = login(t) {
                    connection.close(reason);
                    self.login_failure.push(connection);
                } else {
                    new_connection(connection);
                }

                continue;
            }

            i += 1;
        }

        self.login_failure.retain_mut(|connection| {
            connection.flush();
            !connection.is_closed()
        });
    }
}

pub struct Connection {
    socket: WebSocket<TcpStream>,
}
impl Connection {
    pub fn connect(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(addr).unwrap();
        stream.set_nodelay(true).unwrap();
        stream.set_nonblocking(true).unwrap();

        match tungstenite::client("ws://", stream) {
            Ok((socket, _)) => Self { socket },
            Err(HandshakeError::Interrupted(mut old_mid)) => loop {
                std::thread::sleep(std::time::Duration::from_millis(200));
                match old_mid.handshake() {
                    Ok((socket, _)) => break Self { socket },
                    Err(HandshakeError::Interrupted(mid)) => old_mid = mid,
                    Err(HandshakeError::Failure(err)) => {
                        panic!("Failed to connect: {}", err);
                    }
                }
            },
            Err(HandshakeError::Failure(err)) => {
                panic!("Failed to connect: {}", err);
            }
        }
    }

    pub fn recv<T: Packet>(&mut self) -> Option<T> {
        while let Ok(msg) = self.socket.read() {
            if let Message::Binary(buf) = msg {
                return match T::parse(buf) {
                    Ok(t) => Some(t),
                    Err(err) => {
                        log::debug!("Failed to parse packet: {}", err);
                        None
                    }
                };
            }
        }

        None
    }

    pub fn queue(&mut self, buf: Vec<u8>) {
        let _ = self.socket.write(Message::Binary(buf));
    }

    pub fn flush(&mut self) {
        let _ = self.socket.flush();
    }

    pub fn is_closed(&self) -> bool {
        self.socket.can_read()
    }

    pub fn close(&mut self, reason: &str) {
        let _ = self.socket.close(Some(tungstenite::protocol::CloseFrame {
            code: tungstenite::protocol::frame::coding::CloseCode::Normal,
            reason: reason.into(),
        }));
    }
}
