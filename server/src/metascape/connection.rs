use super::*;
use bytes::{Buf, BufMut};
use futures_util::{SinkExt, StreamExt};
use std::sync::mpsc::TryRecvError;
use tokio_tungstenite::tungstenite::Message;

const ADDR: std::net::SocketAddrV6 =
    std::net::SocketAddrV6::new(std::net::Ipv6Addr::LOCALHOST, 8461, 0, 0);

pub struct Connection {
    pub client_id: ClientId,
    pub disconnected: bool,
    from_client_receiver: std::sync::mpsc::Receiver<Message>,
    to_client_sender: tokio::sync::mpsc::UnboundedSender<Message>,
}
impl Connection {
    pub fn send(&mut self, packet: ServerPacket) {
        self.disconnected |= self.to_client_sender.send(packet.serialize()).is_err();
    }

    pub fn recv(&mut self) -> Option<ClientPacket> {
        match self.from_client_receiver.try_recv() {
            Ok(msg) => {
                let packet = ClientPacket::deserialize(msg);
                self.disconnected |= packet.is_none();
                packet
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.disconnected = true;
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum ServerPacket {
    Fleets {
        time: f64,
        positions: Vec<(FleetId, Vector2<f32>)>,
    },
}
impl ServerPacket {
    fn serialize(self) -> Message {
        let buffer = match self {
            ServerPacket::Fleets { time, positions } => {
                let mut buf = Vec::with_capacity(4 + 8 + 4 + positions.len() * (8 + 4 + 4));

                buf.put_u32_le(0);
                buf.put_f64_le(time);
                buf.put_u32_le(positions.len() as u32);
                for (id, position) in positions {
                    buf.put_u64_le(id.0);
                    buf.put_f32_le(position.x);
                    buf.put_f32_le(position.y);
                }

                buf
            }
        };

        Message::Binary(buffer)
    }
}

#[derive(Debug)]
pub enum ClientPacket {
    MoveFleet {
        fleet_id: FleetId,
        wish_position: Vector2<f32>,
    },
}
impl ClientPacket {
    fn deserialize(msg: Message) -> Option<Self> {
        match msg {
            Message::Text(text) => None,
            Message::Binary(buf) => {
                let mut buf = buf.as_slice();

                let packet_id = buf.get_u32_le();
                match packet_id {
                    0 => {
                        if buf.remaining() < 8 + 4 + 4 {
                            log::debug!("Invalid packet size for MoveFleet");
                            return None;
                        }

                        let fleet_id = FleetId(buf.get_u64_le());
                        let wish_position = Vector2::new(buf.get_f32_le(), buf.get_f32_le());

                        Some(ClientPacket::MoveFleet {
                            fleet_id,
                            wish_position,
                        })
                    }
                    _ => {
                        log::debug!("Invalid packet id {}", packet_id);
                        None
                    }
                }
            }
            Message::Ping(_) => None,
            Message::Pong(_) => None,
            Message::Close(_) => None,
            Message::Frame(_) => None,
        }
    }
}

pub async fn start_server_loop() -> std::sync::mpsc::Receiver<Connection> {
    let listener = tokio::net::TcpListener::bind(ADDR).await.unwrap();
    log::info!("Server bound to {}", listener.local_addr().unwrap());

    let (connection_sender, connection_receiver) = std::sync::mpsc::channel();

    tokio::spawn(async move {
        loop {
            let (stream, addr) = listener.accept().await.unwrap();
            log::debug!("New connection from {}", addr);

            let connection_sender = connection_sender.clone();

            tokio::spawn(async move {
                let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
                    log::debug!("Failed to accept websocket");
                    return;
                };

                let login_packet = match socket.next().await {
                    Some(Ok(msg)) => {
                        if let Message::Text(buf) = msg {
                            if let Some(packet) = LoginPacket::parse(buf) {
                                packet
                            } else {
                                log::debug!("Failed to parse first message");
                                return;
                            }
                        } else {
                            log::debug!("First message was not binary: {:?}", msg);
                            return;
                        }
                    }
                    Some(Err(err)) => {
                        log::debug!("Failed to receive first message: {}", err);
                        return;
                    }
                    None => {
                        log::debug!("Connection closed before first message");
                        return;
                    }
                };
                log::debug!("Parsed login packet: {:?}", login_packet);

                // TODO: Identify client

                let (to_client_sender, mut to_client_receiver) =
                    tokio::sync::mpsc::unbounded_channel();
                let (from_client_sender, from_client_receiver) = std::sync::mpsc::channel();

                let response = LoginResponse {
                    success: true,
                    reason: None,
                };
                log::debug!("Sending login response: {:?}", response);
                to_client_sender
                    .send(Message::Text(serde_json::to_string(&response).unwrap()))
                    .unwrap();

                let connection = Connection {
                    client_id: ClientId(1),
                    disconnected: false,
                    from_client_receiver,
                    to_client_sender,
                };

                connection_sender.send(connection).unwrap();

                let (mut write, mut read) = socket.split();
                tokio::spawn(async move {
                    while let Some(msg) = to_client_receiver.recv().await {
                        if let Err(err) = write.send(msg).await {
                            log::debug!("Failed to send message to client: {}", err);
                            return;
                        }
                    }
                });
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(msg) => {
                            from_client_sender.send(msg).unwrap();
                        }
                        Err(err) => {
                            log::debug!("Failed to receive message from client: {}", err);
                            return;
                        }
                    }
                }
            });
        }
    });

    connection_receiver
}

#[derive(Debug, Deserialize)]
struct LoginPacket {
    username: Option<String>,
    password: Option<String>,
}
impl LoginPacket {
    fn parse(buf: String) -> Option<Self> {
        match serde_json::from_str(&buf) {
            Ok(packet) => Some(packet),
            Err(err) => {
                log::debug!("Failed to parse login packet from '{}': {}", buf, err);
                None
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    success: bool,
    reason: Option<String>,
}
