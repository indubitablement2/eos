use super::*;
use anyhow::Context;
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

pub async fn client_listener(asd: NewClientSenders) {
    let listener = TcpListener::bind("127.0.0.1:32959").await.unwrap();

    match listener.accept().await {
        Ok((stream, _addr)) => {
            tokio::spawn(handle_client(stream));
        }
        Err(err) => log::warn!("failed to accept client: {}", err),
    }
}

async fn handle_client(stream: TcpStream) -> anyhow::Result<()> {
    let mut ws = tokio_tungstenite::accept_async(stream).await?;
    let (tx, mut rx) = ws.split();

    let client_id = loop {
        match rx.next().await.context("ws closed")?? {
            Message::Binary(packet) => {
                // TODO: login with database
                break 0;
            }
            _ => {}
        }
    };

    Ok(())
}
