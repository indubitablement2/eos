use super::*;
use flume::{unbounded, Receiver, Sender, TryRecvError};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    WebSocketStream,
};

pub trait AuthConnection: Send + Clone + 'static {
    type FirstPacket: DeserializeOwned;

    fn auth(
        &self,
        addr: SocketAddr,
        first_packet: Self::FirstPacket,
    ) -> impl std::future::Future<Output = Option<u64>> + std::marker::Send;
}
impl AuthConnection for () {
    type FirstPacket = ();

    async fn auth(&self, _addr: SocketAddr, _first_packet: Self::FirstPacket) -> Option<u64> {
        None
    }
}

#[derive(Clone)]
pub struct ConnectionListener {
    new_connection_receiver: Receiver<(Connection, u64)>,
}
impl ConnectionListener {
    pub fn bind(addr: SocketAddr, auth: impl AuthConnection) -> anyhow::Result<Self> {
        tokio().block_on(Self::bind_async(addr, auth))
    }

    pub async fn bind_async(addr: SocketAddr, auth: impl AuthConnection) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        let (new_connection_sender, new_connection_receiver) = unbounded();

        tokio::spawn(async move {
            let (stream, addr) = match listener.accept().await {
                Ok(ok) => ok,
                Err(err) => {
                    log::error!("Failed to accept connection: {}", err);
                    return;
                }
            };

            let new_connection_sender = new_connection_sender.clone();
            let new_auth = auth.clone();
            tokio::spawn(async move {
                log::debug!("New connection attempt from {}", addr);

                let ws =
                    match tokio_tungstenite::client_async(format!("ws://{}", addr), stream).await {
                        Ok((stream, _)) => stream,
                        Err(err) => {
                            log::debug!("Failed to upgrade connection from {}: {}", addr, err);
                            return;
                        }
                    };

                match Connection::accept(ws, addr, Some(new_auth)).await {
                    Ok(connection) => {
                        if let Err(err) = new_connection_sender.send(connection) {
                            log::debug!("Failed to send connection to main thread: {}", err);
                        }
                    }
                    Err(err) => {
                        log::debug!("Failed to accept connection from {}: {}", addr, err);
                    }
                }
            });

            log::info!("Connection listener closed");
        });

        Ok(Self {
            new_connection_receiver,
        })
    }

    pub fn try_recv(&mut self) -> Option<(Connection, u64)> {
        self.new_connection_receiver.try_recv().ok()
    }
}

enum Outbound {
    Flush,
    Packet(Vec<u8>),
    Close(&'static str),
}

#[derive(Clone)]
pub struct Connection {
    addr: SocketAddr,
    inbound: Receiver<Vec<u8>>,
    outbound: Sender<Outbound>,
}
impl Connection {
    pub fn connect(addr: SocketAddr, login: impl Serialize) -> anyhow::Result<Self> {
        let r = tokio().block_on(Connection::_connect(addr));

        if let Ok(connection) = &r {
            connection.queue(login);
            connection.flush();
        }

        r
    }

    async fn _connect(addr: SocketAddr) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let ws = tokio_tungstenite::client_async(format!("ws://{}", addr), stream)
            .await?
            .0;

        Connection::accept(ws, addr, None::<()>)
            .await
            .map(|(connection, _)| connection)
    }

    async fn accept(
        mut ws: WebSocketStream<TcpStream>,
        addr: SocketAddr,
        auth: Option<impl AuthConnection>,
    ) -> anyhow::Result<(Self, u64)> {
        let _ = ws.get_mut().set_nodelay(true);
        let (mut sink, mut stream) = ws.split();

        let id = if let Some(auth) = auth {
            let first_packet = bin_decode(read_vec(&mut stream).await?.as_slice())?;
            auth.auth(addr, first_packet)
                .await
                .context("Failed to authenticate")?
        } else {
            0
        };

        // Inbound loop
        let (inbound_sender, inbound_receiver) = unbounded();
        let inbound_task = tokio::spawn(async move {
            while let Ok(buf) = read_vec(&mut stream).await {
                if inbound_sender.send(buf).is_err() {
                    break;
                }
            }
        });

        // Outbound loop
        let (outbound_sender, outbound_receiver) = unbounded::<Outbound>();
        tokio::spawn(async move {
            while let Ok(outbound) = outbound_receiver.recv_async().await {
                match outbound {
                    Outbound::Flush => {
                        if let Err(err) = sink.flush().await {
                            log::debug!("Failed to flush packets to {}: {}", addr, err);
                            break;
                        }
                    }
                    Outbound::Packet(buf) => {
                        if let Err(err) = sink.feed(Message::Binary(buf)).await {
                            log::debug!("Failed to feed packet to {}: {}", addr, err);
                            break;
                        }
                    }
                    Outbound::Close(reason) => {
                        let _ = sink
                            .send(Message::Close(Some(CloseFrame {
                                code: CloseCode::Normal,
                                reason: reason.into(),
                            })))
                            .await;
                        break;
                    }
                }
            }

            let _ = sink.close().await;

            inbound_task.abort();

            log::debug!("Connection with {} closed", addr);
        });

        Ok((
            Self {
                addr,
                inbound: inbound_receiver,
                outbound: outbound_sender,
            },
            id,
        ))
    }

    pub fn queue_raw(&self, buf: Vec<u8>) {
        let _ = self.outbound.send(Outbound::Packet(buf));
    }

    pub fn queue(&self, packet: impl Serialize) {
        self.queue_raw(bin_encode(packet));
    }

    pub fn flush(&self) {
        let _ = self.outbound.send(Outbound::Flush);
    }

    pub fn close(&self, reason: &'static str) {
        let _ = self.outbound.send(Outbound::Close(reason));
    }

    pub fn try_recv<T: DeserializeOwned>(&self) -> Option<Result<T, ()>> {
        match self.inbound.try_recv() {
            Ok(buf) => Some(bin_decode(&buf).map_err(|_| ())),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(())),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
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
