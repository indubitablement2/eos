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

pub trait Authentication: Clone + Send + Sized + 'static {
    fn login_packet(
        &mut self,
    ) -> impl std::future::Future<Output = impl Packet> + std::marker::Send;
    fn verify_first_packet(
        &mut self,
        first_packet: Vec<u8>,
    ) -> impl std::future::Future<Output = anyhow::Result<u64>> + std::marker::Send;
}

pub struct ConnectionListener {
    new_connection_receiver: Receiver<(Connection, u64)>,
}
impl ConnectionListener {
    pub fn bind(addr: SocketAddr, auth: impl Authentication) -> Self {
        let listener = tokio()
            .block_on(async move { TcpListener::bind(addr).await })
            .unwrap();

        let (new_connection_sender, new_connection_receiver) = unbounded();

        tokio().spawn(async move {
            loop {
                let (stream, addr) = match listener.accept().await {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::error!("Failed to accept connection: {}", err);
                        break;
                    }
                };

                let auth = auth.clone();
                let new_connection_sender = new_connection_sender.clone();

                tokio::spawn(async move {
                    log::debug!("New connection attempt from {}", addr);

                    match Connection::accept_stream(stream, addr, auth).await {
                        Ok(ok) => {
                            let _ = new_connection_sender.send(ok);
                        }
                        Err(err) => {
                            log::debug!("Failed to accept connection from {}: {}", addr, err);
                        }
                    }
                });
            }
        });

        Self {
            new_connection_receiver,
        }
    }

    pub fn recv(&mut self) -> Option<(Connection, u64)> {
        self.new_connection_receiver.try_recv().ok()
    }
}

pub struct Connection {
    disconnected: bool,
    pub peer_addr: SocketAddr,
    inbound_receiver: Receiver<Vec<u8>>,
    outbound_sender: tokio::sync::mpsc::UnboundedSender<Result<Option<Vec<u8>>, &'static str>>,
}
impl Connection {
    pub fn connect(addr: SocketAddr, auth: impl Authentication) -> anyhow::Result<(Self, u64)> {
        tokio().block_on(Connection::accept_client(addr, auth))
    }

    async fn accept_client(
        server_addr: SocketAddr,
        auth: impl Authentication,
    ) -> anyhow::Result<(Self, u64)> {
        let stream = TcpStream::connect(server_addr).await?;
        let ws = tokio_tungstenite::client_async(format!("ws://{}", server_addr), stream)
            .await?
            .0;
        Connection::accept(ws, server_addr, auth).await
    }

    async fn accept_stream(
        stream: TcpStream,
        addr: SocketAddr,
        auth: impl Authentication,
    ) -> anyhow::Result<(Self, u64)> {
        let ws = tokio_tungstenite::accept_async(stream).await?;
        Connection::accept(ws, addr, auth).await
    }

    async fn accept(
        mut ws: WebSocketStream<TcpStream>,
        addr: SocketAddr,
        mut auth: impl Authentication,
    ) -> anyhow::Result<(Self, u64)> {
        let _ = ws.get_mut().set_nodelay(true);
        let (mut sink, mut stream) = ws.split();

        // Login
        sink.send(Message::Binary(auth.login_packet().await.serialize()))
            .await?;
        let id = match auth.verify_first_packet(read_vec(&mut stream).await?).await {
            Ok(id) => id,
            Err(err) => {
                let _ = sink
                    .send(Message::Close(Some(CloseFrame {
                        code: CloseCode::Normal,
                        reason: err.to_string().into(),
                    })))
                    .await;
                return Err(err);
            }
        };

        // Inbound loop
        let (inbound_sender, inbound_receiver) = unbounded();
        tokio::spawn(async move {
            while let Ok(buf) = read_vec(&mut stream).await {
                if inbound_sender.send(buf).is_err() {
                    break;
                }
            }
            log::debug!("Stream with {} closed", addr);
        });

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

        Ok((
            Self {
                disconnected: false,
                peer_addr: addr,
                inbound_receiver,
                outbound_sender,
            },
            id,
        ))
    }

    pub fn recv<T: Packet>(&mut self) -> Option<T> {
        loop {
            match self.inbound_receiver.try_recv() {
                Ok(buf) => {
                    return match T::parse(buf) {
                        Ok(t) => Some(t),
                        Err(err) => {
                            log::debug!("Failed to parse packet: {}", err);
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
        let _ = self.outbound_sender.send(Ok(Some(packet.serialize())));
    }

    pub fn flush(&self) {
        let _ = self.outbound_sender.send(Ok(None));
    }

    pub fn close(&mut self, reason: &'static str) {
        self.flush();
        self.disconnected = true;
        self.outbound_sender.send(Err(reason)).ok();
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected || self.outbound_sender.is_closed()
    }

    pub fn is_connected(&self) -> bool {
        !self.is_disconnected()
    }
}

async fn read_vec(
    stream: &mut futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<TcpStream>>,
) -> anyhow::Result<Vec<u8>> {
    while let Some(msg) = stream.next().await {
        match msg? {
            Message::Binary(buf) => return Ok(buf),
            Message::Close(_) => anyhow::bail!("Connection closed"),
            _ => {}
        }
    }
    anyhow::bail!("Connection closed");
}
