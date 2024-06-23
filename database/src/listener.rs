use super::*;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub struct Client {
    client_id: ClientId,
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}
impl Client {
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub async fn recv<T: DeserializeOwned>(&mut self) -> Option<T> {
        if let Some(Ok(Message::Binary(data))) = self.ws.next().await {
            bin_decode(&data).ok()
        } else {
            None
        }
    }

    pub async fn send(&mut self, packet: impl Serialize) {
        let _ = self.ws.send(Message::Binary(bin_encode(packet))).await;
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        Database::_client_disconnected(self.client_id);
    }
}

pub async fn listener_loop(addr: impl ToSocketAddrs) {
    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(ok) => ok,
            Err(err) => {
                log::warn!("Failed to accept connection: {}", err);
                continue;
            }
        };

        tokio::spawn(async move {
            log::debug!("New connection attempt from {}", addr);

            if let Err(err) = stream.set_nodelay(true) {
                log::debug!("Failed to set nodelay for {}: {}", addr, err);
            }
            // TODO: Tls
            let stream = MaybeTlsStream::Plain(stream);

            let ws = match tokio_tungstenite::accept_async(stream).await {
                Ok(ws) => ws,
                Err(err) => {
                    log::debug!("Failed to upgrade connection from {}: {}", addr, err);
                    return;
                }
            };

            client::client_loop(ws).await;
        });
    }
}
