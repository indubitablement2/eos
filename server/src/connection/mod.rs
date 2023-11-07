pub mod central_client;
pub mod central_instance;
pub mod client_central;
pub mod client_instance;
pub mod instance_central;
pub mod instance_client;

use super::*;
use bytes::{Buf, BufMut};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{protocol::CloseFrame, Message},
    WebSocketStream,
};

pub trait Packet: Sized {
    fn serialize(self) -> Message;
    fn parse(msg: Message) -> Result<Option<Self>, ()>;
}

/// Dropping this will also cause the inbound side to always recv None.
#[derive(Clone)]
pub struct ConnectionOutbound {
    pub close_reason: &'static str,
    outbound_sender: tokio::sync::mpsc::UnboundedSender<Message>,
}
impl ConnectionOutbound {
    /// Will keep trying to connect until it succeeds.
    pub async fn connect(addr: SocketAddr) -> (Self, ConnectionInbound) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        let stream = loop {
            interval.tick().await;
            if let Ok(stream) = TcpStream::connect(addr).await {
                break stream;
            }
        };

        if let Err(err) = stream.set_nodelay(true) {
            log::warn!("Failed to set nodelay: {}", err);
        }

        let socket = tokio_tungstenite::client_async(format!("ws://{}", addr), stream)
            .await
            .unwrap()
            .0;

        Self::new(socket)
    }

    pub async fn accept(stream: TcpStream) -> Option<(Self, ConnectionInbound)> {
        if let Err(err) = stream.set_nodelay(true) {
            log::warn!("Failed to set nodelay: {}", err);
        }

        let Ok(socket) = tokio_tungstenite::accept_async(stream).await else {
            log::debug!("Failed to accept connection");
            return None;
        };

        Some(Self::new(socket))
    }

    fn new(socket: WebSocketStream<TcpStream>) -> (Self, ConnectionInbound) {
        let (mut outbound, inbound) = socket.split();

        let (outbound_sender, mut outbound_receiver) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(msg) = outbound_receiver.recv().await {
                // TODO: Batch messages.
                if let Err(e) = outbound.send(msg).await {
                    log::debug!("Error sending message: {}", e);
                    break;
                }
            }
        });

        (
            Self {
                close_reason: "Unspecified",
                outbound_sender,
            },
            ConnectionInbound { inbound },
        )
    }

    pub fn send(&self, packet: impl Packet) {
        let _ = self.outbound_sender.send(packet.serialize());
    }

    pub fn send_raw(&self, msg: Message) {
        let _ = self.outbound_sender.send(msg);
    }

    pub fn close(mut self, reason: &'static str) {
        self.close_reason = reason;
    }

    pub fn is_closed(&self) -> bool {
        self.outbound_sender.is_closed()
    }
}
impl Drop for ConnectionOutbound {
    fn drop(&mut self) {
        let _ = self.outbound_sender.send(Message::Close(Some(CloseFrame {
            code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
            reason: self.close_reason.into(),
        })));
    }
}

pub struct ConnectionInbound {
    inbound: futures_util::stream::SplitStream<WebSocketStream<TcpStream>>,
}
impl ConnectionInbound {
    /// Will return `None` if the outbound side is dropped.
    pub async fn recv<T: Packet>(&mut self) -> Option<T> {
        while let Some(msg) = self.inbound.next().await {
            match msg {
                Ok(msg) => match T::parse(msg) {
                    Ok(Some(t)) => return Some(t),
                    Ok(None) => continue,
                    Err(_) => {
                        log::debug!("Failed to parse message");
                        return None;
                    }
                },
                Err(err) => {
                    log::debug!("Error receiving message: {}", err);
                    return None;
                }
            }
        }
        None
    }
}

trait VariantEncoding: BufMut {
    /// Advance by 8.
    fn put_bool_var(&mut self, value: bool) {
        self.put_u32_le(1);
        self.put_u32_le(value as u32);
    }

    /// Advance by 8.
    fn put_u32_var(&mut self, value: u32) {
        self.put_u32_le(2);
        self.put_u32_le(value);
    }

    /// Advance by 12.
    fn put_u64_var(&mut self, value: u64) {
        self.put_u32_le(2 | (1 << 16));
        self.put_u64_le(value);
    }

    /// Advance by 8.
    fn put_f32_var(&mut self, value: f32) {
        self.put_u32_le(3);
        self.put_f32_le(value);
    }

    /// Advance by 12.
    fn put_f64_var(&mut self, value: f64) {
        self.put_u32_le(3 | (1 << 16));
        self.put_f64_le(value);
    }

    /// Advance by 8 + bytes (padded to 4).
    fn put_string_var(&mut self, value: &str) {
        self.put_u32_le(4);
        self.put_u32_le(value.len() as u32);
        self.put_slice(value.as_bytes());
        let padding = value.len().next_multiple_of(4) - value.len();
        self.put_bytes(0, padding);
    }

    /// Advance by 12.
    fn put_vec2_var(&mut self, value: Vector2<f32>) {
        self.put_u32_le(5);
        self.put_f32_le(value.x);
        self.put_f32_le(value.y);
    }

    /// Advance by 8.
    fn put_array_var(&mut self, len: usize) {
        self.put_u32_le(28);
        self.put_u32_le(len as u32);
    }

    /// Advance by 8 + bytes (padded to 4).
    fn put_bytes_var(&mut self, value: &[u8]) {
        self.put_u32_le(29);
        self.put_u32_le(value.len() as u32);
        self.put_slice(value);
        let padding = value.len().next_multiple_of(4) - value.len();
        self.put_bytes(0, padding);
    }
}
impl VariantEncoding for Vec<u8> {}

trait VariantDecoding: Buf {
    fn get_bool_var(&mut self) -> Result<bool, ()> {
        if self.remaining() < 8 {
            return Err(());
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 1 && flag == 0);

        Ok(self.get_u32_le() != 0)
    }

    /// Convert to u32 if needed.
    fn get_u32_var(&mut self) -> Result<u32, ()> {
        if self.remaining() < 8 {
            return Err(());
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 2);

        if flag == 0 {
            if self.remaining() < 4 {
                Err(())
            } else {
                Ok(self.get_u32_le())
            }
        } else {
            if self.remaining() < 8 {
                Err(())
            } else {
                Ok(self.get_u64_le() as u32)
            }
        }
    }

    /// Convert to u32 if needed.
    fn get_u64_var(&mut self) -> Result<u64, ()> {
        if self.remaining() < 8 {
            return Err(());
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 2);

        if flag == 0 {
            if self.remaining() < 4 {
                Err(())
            } else {
                Ok(self.get_u32_le() as u64)
            }
        } else {
            if self.remaining() < 8 {
                Err(())
            } else {
                Ok(self.get_u64_le())
            }
        }
    }

    /// Convert f64 to f32 if needed.
    fn get_f32_var(&mut self) -> Result<f32, ()> {
        if self.remaining() < 8 {
            return Err(());
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 3);

        if flag == 0 {
            if self.remaining() < 4 {
                Err(())
            } else {
                Ok(self.get_f32_le())
            }
        } else {
            if self.remaining() < 8 {
                Err(())
            } else {
                Ok(self.get_f64_le() as f32)
            }
        }
    }

    /// Convert f32 to f64 if needed.
    fn get_f64_var(&mut self) -> Result<f64, ()> {
        if self.remaining() < 8 {
            return Err(());
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 3);

        if flag == 0 {
            if self.remaining() < 4 {
                Err(())
            } else {
                Ok(self.get_f32_le() as f64)
            }
        } else {
            if self.remaining() < 8 {
                Err(())
            } else {
                Ok(self.get_f64_le())
            }
        }
    }

    fn get_string_var(&mut self) -> Result<String, ()> {
        if self.remaining() < 8 {
            return Err(());
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 4 && flag == 0);

        let len = self.get_u32_le() as usize;
        let full_len = len.next_multiple_of(4);
        if self.remaining() < full_len {
            log::debug!(
                "Buffer too small for string of length {}( has:{}, need:{})",
                len,
                self.remaining(),
                full_len
            );
            return Err(());
        }

        let mut vec = vec![0; full_len];
        self.copy_to_slice(&mut vec);
        vec.truncate(len);
        String::from_utf8(vec).map_err(|err| {
            log::debug!("Failed to parse string: {}", err);
            ()
        })
    }

    fn get_vec2_var(&mut self) -> Result<Vector2<f32>, ()> {
        if self.remaining() < 12 {
            return Err(());
        }

        let header = self.get_u32_le();
        let t = header & 0xFFFF;
        let flag = header >> 16;
        debug_assert!(t == 5 && flag == 0);

        Ok(Vector2::new(self.get_f32_le(), self.get_f32_le()))
    }

    fn get_array_var(&mut self) -> Result<usize, ()> {
        if self.remaining() < 8 {
            return Err(());
        }

        // Header has godot properties which we don't care about.
        self.advance(4);

        Ok(self.get_u32_le() as usize)
    }
}
impl VariantDecoding for &[u8] {}
