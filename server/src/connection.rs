use std::net::SocketAddr;

use bytes::{Buf, BufMut};
use futures_util::{SinkExt, StreamExt};
use tokio_util::codec::{Decoder, Encoder};

/// Address for the instance servers to connect to.
const CENTRAL_ADDR_INSTANCE: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
    std::net::Ipv6Addr::LOCALHOST,
    12461,
    0,
    0,
));
/// Address for the clients to connect to.
const CENTRAL_ADDR_CLIENT: SocketAddr = SocketAddr::V6(std::net::SocketAddrV6::new(
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

pub struct Connection<Inbound, Outbound>
where
    Inbound: DeserializePacket,
    Outbound: SerializePacket,
{
    pub address: SocketAddr,

    pub disconnected: bool,

    inbound_receiver: std::sync::mpsc::Receiver<Vec<u8>>,
    outbound_sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,

    _phantom: std::marker::PhantomData<(Inbound, Outbound)>,
}
impl<Inbound, Outbound> Connection<Inbound, Outbound>
where
    Inbound: DeserializePacket + Send + 'static,
    Outbound: SerializePacket + Send + 'static,
{
    pub async fn bind_central_client() -> std::sync::mpsc::Receiver<Self> {
        let (new_connection_sender, new_connection_receiver) = std::sync::mpsc::channel();

        let listener = tokio::net::TcpListener::bind(CENTRAL_ADDR_CLIENT)
            .await
            .unwrap();
        log::info!(
            "Central to client bound to {}",
            listener.local_addr().unwrap()
        );

        tokio::spawn(async move {
            while let Ok((stream, address)) = listener.accept().await {
                log::debug!("New connection from client {}", address);

                if let Some(connection) = Self::new_central_client(stream, address).await {
                    if new_connection_sender.send(connection).is_err() {
                        log::debug!("Failed to accept client websocket");
                        break;
                    }
                }
            }
        });

        new_connection_receiver
    }

    async fn new_central_client(
        stream: tokio::net::TcpStream,
        address: SocketAddr,
    ) -> Option<Self> {
        let Ok(socket) = tokio_tungstenite::accept_async(stream).await else {
            log::debug!("Failed to accept websocket");
            return None;
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

        Some(Self {
            address,
            disconnected: false,
            inbound_receiver,
            outbound_sender,
            _phantom: std::marker::PhantomData,
        })
    }

    pub async fn bind_central_instance() -> std::sync::mpsc::Receiver<Self> {
        let (new_connection_sender, new_connection_receiver) = std::sync::mpsc::channel();

        let listener = tokio::net::TcpListener::bind(CENTRAL_ADDR_INSTANCE)
            .await
            .unwrap();

        tokio::spawn(async move {
            while let Ok((stream, address)) = listener.accept().await {
                let connection = Self::new_central_instance(stream, address).await;
                if new_connection_sender.send(connection).is_err() {
                    break;
                }
            }
        });

        new_connection_receiver
    }

    async fn new_central_instance(stream: tokio::net::TcpStream, address: SocketAddr) -> Self {
        let (outbound_sender, mut outbound_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        let (inbound_sender, inbound_receiver) = std::sync::mpsc::channel();

        let (r, w) = stream.into_split();
        let mut framed_r = tokio_util::codec::FramedRead::new(r, SimpleInstanceDecoder::default());
        let mut framed_w = tokio_util::codec::FramedWrite::new(w, SimpleInstanceEncoder);

        tokio::spawn(async move {
            while let Some(packet) = framed_r.next().await {
                if inbound_sender.send(packet.unwrap()).is_err() {
                    break;
                }
            }
        });
        tokio::spawn(async move {
            while let Some(packet) = outbound_receiver.recv().await {
                if framed_w.send(packet.as_slice()).await.is_err() {
                    break;
                }
            }
        });

        Self {
            address,
            disconnected: false,
            inbound_receiver,
            outbound_sender,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn connect_instance_to_central() -> Self {
        let (outbound_sender, mut outbound_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        let (inbound_sender, inbound_receiver) = std::sync::mpsc::channel();

        let (r, w) = tokio::net::TcpSocket::new_v6()
            .unwrap()
            .connect(CENTRAL_ADDR_INSTANCE)
            .await
            .unwrap()
            .into_split();

        let mut framed_r = tokio_util::codec::FramedRead::new(r, SimpleInstanceDecoder::default());
        let mut framed_w = tokio_util::codec::FramedWrite::new(w, SimpleInstanceEncoder);

        tokio::spawn(async move {
            while let Some(packet) = framed_r.next().await {
                if inbound_sender.send(packet.unwrap()).is_err() {
                    break;
                }
            }
        });
        tokio::spawn(async move {
            while let Some(packet) = outbound_receiver.recv().await {
                if framed_w.send(packet.as_slice()).await.is_err() {
                    break;
                }
            }
        });

        Self {
            address: CENTRAL_ADDR_INSTANCE,
            disconnected: false,
            inbound_receiver,
            outbound_sender,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn recv(&mut self) -> Option<Inbound> {
        match self.inbound_receiver.try_recv() {
            Ok(packet) => {
                let result = Inbound::deserialize(&packet);
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

    pub fn send(&mut self, packet: Outbound) {
        self.disconnected |= self.outbound_sender.send(packet.serialize()).is_err()
    }
}

#[derive(Default)]
struct SimpleInstanceDecoder {
    next_len: Option<u32>,
}
impl Decoder for SimpleInstanceDecoder {
    type Item = Vec<u8>;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.next_len.is_none() && src.len() >= 4 {
            self.next_len = Some(src.get_u32_le());
        }

        if let Some(next_len) = self.next_len {
            if src.len() >= next_len as usize {
                let packet = src.split_to(next_len as usize);
                self.next_len = None;
                Ok(Some(packet.to_vec()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

struct SimpleInstanceEncoder;
impl Encoder<&[u8]> for SimpleInstanceEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &[u8], dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        dst.put_u32_le(item.len() as u32);
        dst.put(item);
        Ok(())
    }
}
