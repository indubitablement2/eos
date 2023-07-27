use super::*;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpStream, UdpSocket},
    spawn,
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub struct ClientConnection {
    pub client_id: ClientId,
    pub addr: SocketAddr,
    to_client_reliable_sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    to_client_unreliable_sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    from_client_receiver: crossbeam::channel::Receiver<Vec<u8>>,
    pub disconnected: bool,
}
impl ClientConnection {
    pub fn send(&mut self, packet: ToClientPacket) {
        let reliable = packet.reliable();
        let buf = rmp_serde::to_vec(&packet).expect("packet should serialize");
        let result = if reliable || buf.len() > 1024 {
            self.to_client_reliable_sender.send(buf)
        } else {
            self.to_client_unreliable_sender.send(buf)
        };

        if result.is_err() {
            self.disconnected = true;
        }
    }

    pub fn receive(&mut self) -> Option<FromClientPacket> {
        match self.from_client_receiver.try_recv() {
            Ok(bytes) => match serde_json::from_slice(bytes.as_slice()) {
                Ok(packet) => Some(packet),
                Err(err) => {
                    log::debug!("failed to deserialize packet from client: {:?}", err);
                    self.disconnected = true;
                    None
                }
            },
            Err(crossbeam::channel::TryRecvError::Disconnected) => {
                self.disconnected = true;
                None
            }
            Err(crossbeam::channel::TryRecvError::Empty) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ToClientPacket {
    SayHello,
    Update,
}
impl ToClientPacket {
    pub fn reliable(&self) -> bool {
        match self {
            ToClientPacket::SayHello => true,
            ToClientPacket::Update => false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FromClientPacket {
    //
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct FromClientFirstPacket {
    wish_udp: bool,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "snake_case")]
struct ToClientFirstPacket {
    server_upd_full_addr: Option<SocketAddr>,
    server_udp_port: u16,
}

pub async fn init_client_connection(stream: TcpStream) -> Result<ClientConnection> {
    stream.set_nodelay(false)?;

    let addr = stream.peer_addr()?;
    debug!("New tcp connection established with: {}", addr);

    let stream = tokio_tungstenite::accept_async(stream).await?;
    debug!("New websocket connection established with: {}", addr);

    let (mut s, mut r) = stream.split();

    // TODO: Encryption
    // TODO: Identify client.
    let client_id = ClientId(0);

    let first = r.next().await.ok_or(anyhow!("client disconnected"))??;
    let first: FromClientFirstPacket = serde_json::from_slice(first.into_data().as_slice())?;
    debug!("Received first packet from client: {:?}", first);

    let mut to_client_first_packet = ToClientFirstPacket::default();

    let (from_client_sender, from_client_receiver) = crossbeam::channel::unbounded();
    let (to_client_reliable_sender, to_client_reliable_receiver) =
        tokio::sync::mpsc::unbounded_channel();

    let to_client_unreliable_sender = if first.wish_udp {
        let udp = Arc::new(tokio::net::UdpSocket::bind("::").await?);
        udp.connect(addr).await?;

        let server_addr = udp.local_addr()?;
        to_client_first_packet.server_udp_port = server_addr.port();
        to_client_first_packet.server_upd_full_addr = Some(server_addr);

        let (to_client_unreliable_sender, to_client_unreliable_receiver) =
            tokio::sync::mpsc::unbounded_channel();

        // Start udp loops.
        spawn(to_client_unreliable(
            udp.clone(),
            to_client_unreliable_receiver,
        ));
        spawn(from_client_unreliable(udp, from_client_sender.clone()));

        debug!("UDP connection established with: {}", addr);

        to_client_unreliable_sender
    } else {
        // No udp connection. Forward all packets to tcp.
        to_client_reliable_sender.clone()
    };

    debug!(
        "Sending first packet to client: {:?}",
        to_client_first_packet
    );
    s.send(Message::Binary(serde_json::to_vec(
        &to_client_first_packet,
    )?))
    .await?;

    // Start tcp loops.
    spawn(to_client_reliable(s, to_client_reliable_receiver));
    spawn(from_client_reliable(r, from_client_sender));

    debug!("Client connection initialized: {}", addr);

    Ok(ClientConnection {
        client_id,
        addr,
        to_client_reliable_sender,
        to_client_unreliable_sender,
        from_client_receiver,
        disconnected: false,
    })
}

async fn to_client_reliable(
    mut tcp_sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut to_client_reliable_receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
) -> Result<()> {
    while let Some(packet) = to_client_reliable_receiver.recv().await {
        tcp_sender.send(Message::Binary(packet)).await?;
    }

    Ok(())
}

async fn to_client_unreliable(
    udp: Arc<UdpSocket>,
    mut to_client_unreliable_receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
) -> Result<()> {
    while let Some(packet) = to_client_unreliable_receiver.recv().await {
        udp.send(packet.as_slice()).await?;
    }

    Ok(())
}

async fn from_client_reliable(
    mut tcp_receiver: SplitStream<WebSocketStream<TcpStream>>,
    from_client_sender: crossbeam::channel::Sender<Vec<u8>>,
) -> Result<()> {
    while let Some(msg) = tcp_receiver.next().await {
        from_client_sender.send(msg?.into_data())?;
    }

    Ok(())
}

async fn from_client_unreliable(
    udp: Arc<UdpSocket>,
    from_client_sender: crossbeam::channel::Sender<Vec<u8>>,
) -> Result<()> {
    let mut buf = vec![0; 1200];
    while let Ok(len) = udp.recv(&mut buf).await {
        from_client_sender.send(buf[..len].to_vec())?;
    }

    Ok(())
}

#[test]
fn asd() {
    #[derive(Debug, Serialize, Deserialize)]
    struct Test {
        a: u8,
        b: u16,
        s: String,
        vec: Vec<u8>,
        vec2: Vector2<f32>,
    }

    let test = Test {
        a: 250,
        b: 50000,
        s: "asd".to_string(),
        vec: vec![1, 2, 3, 255],
        vec2: Vector2::new(1.0, 2.0),
    };

    let buf = rmp_serde::to_vec(&test).unwrap();
    println!("{:?}", buf);
    let buf = rmp_serde::to_vec(&Vector2::new(1.0f32, 2.0)).unwrap();
    println!("{:?}", buf);
    let buf = rmp_serde::to_vec(&1.0f32).unwrap();
    println!("{:?}", buf);
    let buf = rmp_serde::to_vec(&50000u16).unwrap();
    println!("{:?}", buf);
}
