use super::*;
use flume::{unbounded, Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_tungstenite::{
    tungstenite::{client::IntoClientRequest, Message},
    MaybeTlsStream, WebSocketStream,
};

static BYTES_IN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PACKETS_IN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

static BYTES_OUT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PACKETS_OUT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Clone)]
pub struct ConnectionListener {
    new_connection_receiver: Receiver<Connection>,
}
impl ConnectionListener {
    pub fn bind(addr: impl ToSocketAddrs) -> anyhow::Result<Self> {
        tokio().block_on(Self::bind_async(addr))
    }

    pub async fn bind_async(addr: impl ToSocketAddrs) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        let (new_connection_sender, new_connection_receiver) = unbounded();

        tokio::spawn(async move {
            // TODO: Tls
            loop {
                let (stream, addr) = match listener.accept().await {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::error!("Failed to accept connection: {}", err);
                        continue;
                    }
                };
                if let Err(err) = stream.set_nodelay(true) {
                    log::debug!("Failed to set nodelay: {}", err);
                }
                let stream = MaybeTlsStream::Plain(stream);

                let new_connection_sender = new_connection_sender.clone();
                tokio::spawn(async move {
                    log::debug!("New connection attempt from {}", addr);

                    let ws = match tokio_tungstenite::accept_async(stream).await {
                        Ok(ws) => ws,
                        Err(err) => {
                            log::debug!("Failed to upgrade connection from {}: {}", addr, err);
                            return;
                        }
                    };

                    match Connection::accept(ws).await {
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
            }
        });

        Ok(Self {
            new_connection_receiver,
        })
    }

    pub fn try_recv(&self) -> Option<Connection> {
        self.new_connection_receiver.try_recv().ok()
    }
}

enum Outbound {
    Flush,
    Packet(Vec<u8>),
    Close,
}

/// Automatically closes the connection when all connections are dropped.
#[derive(Clone)]
pub struct Connection {
    inbound: Receiver<Vec<u8>>,
    outbound: Sender<Outbound>,
}
impl Connection {
    pub fn packets_out() -> u64 {
        PACKETS_OUT.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn bytes_out() -> u64 {
        BYTES_OUT.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn packets_in() -> u64 {
        PACKETS_IN.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn bytes_in() -> u64 {
        BYTES_IN.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn connect(request: impl IntoClientRequest) -> anyhow::Result<Self> {
        tokio().block_on(Self::connect_async(request))
    }

    pub async fn connect_async(request: impl IntoClientRequest) -> anyhow::Result<Self> {
        let request = request.into_client_request()?;
        let ws = tokio_tungstenite::connect_async_with_config(request, None, true)
            .await?
            .0;

        Connection::accept(ws).await
    }

    async fn accept(ws: WebSocketStream<MaybeTlsStream<TcpStream>>) -> anyhow::Result<Self> {
        let (mut sink, mut stream) = ws.split();

        // Inbound loop
        let (inbound_sender, inbound_receiver) = unbounded();
        let inbound_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                PACKETS_IN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                match msg {
                    Message::Binary(buf) => {
                        BYTES_IN.fetch_add(buf.len() as u64, std::sync::atomic::Ordering::Relaxed);

                        if inbound_sender.send(buf).is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        });

        // Outbound loop
        let (outbound_sender, outbound_receiver) = unbounded::<Outbound>();
        tokio::spawn(async move {
            while let Ok(outbound) = outbound_receiver.recv_async().await {
                match outbound {
                    Outbound::Flush => {
                        PACKETS_OUT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        if let Err(err) = sink.flush().await {
                            log::debug!("Failed to flush packets: {}", err);
                            break;
                        }
                    }
                    Outbound::Packet(buf) => {
                        BYTES_OUT.fetch_add(buf.len() as u64, std::sync::atomic::Ordering::Relaxed);

                        if let Err(err) = sink.feed(Message::Binary(buf)).await {
                            log::debug!("Failed to feed packet: {}", err);
                            break;
                        }
                    }
                    Outbound::Close => {
                        break;
                    }
                }
            }

            if let Err(err) = sink.close().await {
                log::debug!("Failed to close sink: {}", err);
            }

            inbound_task.abort();

            log::debug!("Connection closed");
        });

        Ok(Self {
            inbound: inbound_receiver,
            outbound: outbound_sender,
        })
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

    pub fn close(&self) {
        let _ = self.outbound.send(Outbound::Close);
    }

    pub fn try_recv_raw(&self) -> Option<Vec<u8>> {
        self.inbound.try_recv().ok()
    }

    pub fn block_recv_raw(&self) -> Option<Vec<u8>> {
        self.inbound.recv().ok()
    }

    pub fn try_recv<P: DeserializeOwned>(&self) -> Option<P> {
        self.try_recv_raw().and_then(|buf| bin_decode(&buf).ok())
    }

    pub fn block_recv<P: DeserializeOwned>(&self) -> Option<P> {
        self.block_recv_raw().and_then(|buf| bin_decode(&buf).ok())
    }

    pub fn is_closed(&self) -> bool {
        self.inbound.is_disconnected()
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        if self.inbound.receiver_count() == 1 {
            self.close();
        }
    }
}
