use std::sync::{atomic::AtomicBool, Arc};

use super::*;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    WebSocketStream,
};

pub trait Packet: Sized + Send {
    fn serialize(self) -> Vec<u8>;
    fn parse(buf: Vec<u8>) -> anyhow::Result<Self>;
}
impl Packet for Vec<u8> {
    fn serialize(self) -> Vec<u8> {
        self
    }

    fn parse(buf: Vec<u8>) -> anyhow::Result<Self> {
        Ok(buf)
    }
}

pub struct ConnectionListener<T: Packet> {
    new_connection_receiver: Receiver<(Connection, T)>,
}
impl<T: Packet + 'static> ConnectionListener<T> {
    pub fn bind(addr: SocketAddr) -> Self {
        let listener = tokio()
            .block_on(async move { TcpListener::bind(addr).await })
            .unwrap();

        let (new_connection_sender, new_connection_receiver) = unbounded();

        tokio().spawn(async move {
            let closed = Arc::new(AtomicBool::new(false));
            while !closed.load(std::sync::atomic::Ordering::Relaxed) {
                let (stream, addr) = match listener.accept().await {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::error!("Failed to accept connection: {}", err);
                        break;
                    }
                };

                let new_connection_sender = new_connection_sender.clone();
                let closed = closed.clone();
                tokio::spawn(async move {
                    log::debug!("New connection attempt from {}", addr);

                    match Connection::accept_stream::<T>(stream, addr).await {
                        Ok((connection, login, stream, inbound_sender)) => {
                            if new_connection_sender.send((connection, login)).is_err() {
                                closed.store(true, std::sync::atomic::Ordering::Relaxed);
                            }

                            Connection::inbound_loop(stream, inbound_sender, addr).await;
                        }
                        Err(err) => {
                            log::debug!("Failed to accept connection from {}: {}", addr, err);
                        }
                    }
                });
            }

            log::debug!("Connection listener closed");
        });

        Self {
            new_connection_receiver,
        }
    }

    pub fn recv(&mut self) -> Option<(Connection, T)> {
        self.new_connection_receiver.try_recv().ok()
    }
}

#[derive(Clone)]
pub struct ConnectionOutbound {
    outbound_sender: tokio::sync::mpsc::UnboundedSender<Result<Option<Vec<u8>>, &'static str>>,
}
impl ConnectionOutbound {
    pub fn queue(&self, packet: impl Packet) {
        let _ = self.outbound_sender.send(Ok(Some(packet.serialize())));
    }

    pub fn flush(&self) {
        let _ = self.outbound_sender.send(Ok(None));
    }

    pub fn is_disconnected(&self) -> bool {
        self.outbound_sender.is_closed()
    }

    pub fn is_connected(&self) -> bool {
        !self.is_disconnected()
    }
}

pub struct Connection {
    disconnected: bool,
    pub peer_addr: SocketAddr,
    inbound_receiver: Receiver<Vec<u8>>,
    pub connection_outbound: ConnectionOutbound,
}
impl Connection {
    pub fn connect(addr: SocketAddr, login: impl Packet) -> anyhow::Result<Self> {
        let r = tokio().block_on(Connection::accept_client(addr));

        if let Ok(connection) = &r {
            connection.queue(login);
            connection.flush();
        }

        r
    }

    pub fn split(self) -> (ConnectionOutbound, Receiver<Vec<u8>>) {
        (self.connection_outbound, self.inbound_receiver)
    }

    async fn accept_client(server_addr: SocketAddr) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(server_addr).await?;
        let ws = tokio_tungstenite::client_async(format!("ws://{}", server_addr), stream)
            .await?
            .0;

        let (connection, stream, inbound_sender) = Connection::accept(ws, server_addr).await?;
        let addr = connection.peer_addr;
        tokio::spawn(Connection::inbound_loop(stream, inbound_sender, addr));

        Ok(connection)
    }

    async fn accept_stream<T: Packet>(
        stream: TcpStream,
        addr: SocketAddr,
    ) -> anyhow::Result<(Self, T, WsStream, Sender<Vec<u8>>)> {
        let ws = tokio_tungstenite::accept_async(stream).await?;
        let (connection, mut stream, inbound_sender) = Connection::accept(ws, addr).await?;

        let login = T::parse(read_vec(&mut stream).await?)?;

        Ok((connection, login, stream, inbound_sender))
    }

    async fn accept(
        mut ws: WebSocketStream<TcpStream>,
        addr: SocketAddr,
    ) -> anyhow::Result<(Self, WsStream, Sender<Vec<u8>>)> {
        let _ = ws.get_mut().set_nodelay(true);
        let (mut sink, stream) = ws.split();

        // Outbound loop
        let (outbound_sender, mut outbound_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Result<Option<Vec<u8>>, &'static str>>();
        tokio::spawn(async move {
            while let Some(buf) = outbound_receiver.recv().await {
                match buf {
                    Ok(Some(buf)) => {
                        if let Err(err) = sink.feed(Message::Binary(buf)).await {
                            log::debug!("Failed to feed packet to {}: {}", addr, err);
                            break;
                        }
                    }
                    Ok(None) => {
                        if let Err(err) = sink.flush().await {
                            log::debug!("Failed to flush packets to {}: {}", addr, err);
                            break;
                        }
                    }
                    Err(close_reason) => {
                        let _ = sink
                            .send(Message::Close(Some(CloseFrame {
                                code: CloseCode::Normal,
                                reason: close_reason.into(),
                            })))
                            .await;
                        break;
                    }
                }
            }

            log::debug!("Sink with {} closed", addr);
        });

        let (inbound_sender, inbound_receiver) = unbounded();

        Ok((
            Self {
                disconnected: false,
                peer_addr: addr,
                inbound_receiver,
                connection_outbound: ConnectionOutbound { outbound_sender },
            },
            stream,
            inbound_sender,
        ))
    }

    async fn inbound_loop(mut stream: WsStream, inbound_sender: Sender<Vec<u8>>, addr: SocketAddr) {
        while let Ok(buf) = read_vec(&mut stream).await {
            if inbound_sender.send(buf).is_err() {
                break;
            }
        }
        log::debug!("Stream with {} closed", addr);
    }

    pub fn recv<T: Packet>(&mut self) -> Option<T> {
        loop {
            match self.inbound_receiver.try_recv() {
                Ok(buf) => {
                    match T::parse(buf) {
                        Ok(t) => return Some(t),
                        Err(err) => {
                            log::debug!("Failed to parse packet: {}", err);
                            self.disconnected = true;
                            return None;
                        }
                    };
                }
                Err(TryRecvError::Empty) => {
                    return None;
                }
                Err(TryRecvError::Disconnected) => {
                    self.disconnected = true;
                    return None;
                }
            }
        }
    }

    pub fn queue(&self, packet: impl Packet) {
        self.connection_outbound.queue(packet);
    }

    pub fn flush(&self) {
        self.connection_outbound.flush();
    }

    pub fn close(&mut self, reason: &'static str) {
        self.disconnected = true;
        self.connection_outbound
            .outbound_sender
            .send(Err(reason))
            .ok();
        self.flush();
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected || self.connection_outbound.is_disconnected()
    }

    pub fn is_connected(&self) -> bool {
        !self.is_disconnected()
    }
}

type WsStream = futures_util::stream::SplitStream<WebSocketStream<TcpStream>>;

async fn read_vec(stream: &mut WsStream) -> anyhow::Result<Vec<u8>> {
    while let Some(msg) = stream.next().await {
        match msg? {
            Message::Binary(buf) => return Ok(buf),
            Message::Close(_) => anyhow::bail!("Connection closed"),
            _ => {}
        }
    }
    anyhow::bail!("Connection closed");
}
